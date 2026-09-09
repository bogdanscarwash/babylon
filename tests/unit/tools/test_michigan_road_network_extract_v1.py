"""The real streaming adapter retains complete ways, bounds selection and refuses faults."""

import hashlib
import json
import sqlite3

import osmium
import pytest
import tools.michigan_road_network_extract_v1 as extraction
import tools.michigan_road_network_v1 as network
from tools.michigan_road_network_extract_v1 import Footprint, IdBloom, RoadSpool, extract_pbf


def footprint(tmp_path):
    path = tmp_path / "footprint.json"
    path.write_text(
        json.dumps(
            {
                "type": "Polygon",
                "coordinates": [
                    [
                        [-85.001, 43.999],
                        [-84.999, 43.999],
                        [-84.999, 44.001],
                        [-85.001, 44.001],
                        [-85.001, 43.999],
                    ]
                ],
            }
        )
    )
    return Footprint(path, 0)


def pbf(tmp_path):
    xml = tmp_path / "source.osm"
    xml.write_text("""<osm version="0.6">
      <node id="1" version="1" lat="44" lon="-85"/>
      <node id="2" version="1" lat="44" lon="-84.998"/>
      <node id="3" version="1" lat="44" lon="-84.997"/>
      <node id="4" version="1" lat="44.02" lon="-85"/>
      <node id="5" version="1" lat="44.03" lon="-85"/>
      <way id="10" version="1"><nd ref="1"/><nd ref="2"/><tag k="highway" v="primary"/></way>
      <way id="20" version="1"><nd ref="2"/><nd ref="3"/><tag k="highway" v="primary"/></way>
      <way id="30" version="1"><nd ref="4"/><nd ref="5"/><tag k="highway" v="primary"/></way>
      <relation id="100" version="1"><member type="way" ref="10" role="from"/><member type="node" ref="2" role="via"/><member type="way" ref="20" role="to"/><tag k="type" v="restriction"/><tag k="restriction" v="only_straight_on"/></relation>
    </osm>""")
    path = tmp_path / "source.osm.pbf"
    header = osmium.io.Header()
    header.set("osmosis_replication_timestamp", "2026-09-08T20:21:01Z")
    with osmium.SimpleWriter(str(path), header=header) as writer:
        for row in osmium.FileProcessor(str(xml)):
            writer.add(row)
    return path


def test_real_pbf_adapter_completes_way_nodes_and_retains_boundary_restriction(tmp_path):
    spool = RoadSpool(tmp_path / "spool.sqlite", bloom_bytes=1024)
    try:
        extract_pbf(pbf(tmp_path), footprint(tmp_path), spool, progress=lambda _: None)
        assert [row.id for row in spool.nodes()] == [1, 2]
        assert [row.id for row in spool.ways()] == [10]
        assert [row.id for row in spool.restrictions()] == [100]
        source = network.SourceIdentity(
            "a" * 64, 1, "fixture", "fixture", "b" * 64, 0, "fixture", "fixture"
        )
        roads = network.build_graph(
            spool.nodes(),
            spool.ways(),
            spool.restrictions(),
            source=source,
            profile=network.VehicleProfile(40000, 4000, 2550, 16500, 4000),
        )
        assert not roads.edges
        assert any(
            item.code == "unsupported_restriction" and "outside complete extract" in item.detail
            for item in roads.diagnostics
        )
    finally:
        spool.close()


def test_bloom_collisions_cannot_include_an_outside_way(tmp_path):
    spool = RoadSpool(tmp_path / "spool.sqlite", bloom_bytes=1)
    try:
        spool.add_footprint_node(network.Node(1, 0, 0))
        spool.flush_nodes()
        spool.footprint_ids.bits[:] = b"\xff"
        spool.add_touching_way(network.Way(99, (2, 3), (("highway", "primary"),)))
        assert list(spool.ways()) == []
    finally:
        spool.close()


def test_progress_reader_cannot_interrupt_spool_commits(tmp_path):
    path = tmp_path / "spool.sqlite"
    spool = RoadSpool(path, bloom_bytes=1024)
    reader = sqlite3.connect(path)
    try:
        spool.add_footprint_node(network.Node(1, 0, 0))
        spool.flush_nodes()
        reader.execute("BEGIN")
        assert reader.execute("SELECT count(*) FROM nodes").fetchone()[0] == 1
        spool.database.execute("PRAGMA busy_timeout=50")
        spool.add_footprint_node(network.Node(2, 1, 1))
        spool.flush_nodes()
        assert reader.execute("SELECT count(*) FROM nodes").fetchone()[0] == 1
        reader.commit()
        assert reader.execute("SELECT count(*) FROM nodes").fetchone()[0] == 2
    finally:
        reader.close()
        spool.close()


def test_missing_osmium_is_a_specific_refusal(monkeypatch):
    def missing(_):
        raise ModuleNotFoundError("osmium")

    monkeypatch.setattr(extraction.importlib, "import_module", missing)
    with pytest.raises(network.RoadNetworkError, match="pinned project 'osmium'"):
        extraction.require_osmium()


def test_extraction_missing_node_and_existing_spool_refuse(tmp_path):
    path = tmp_path / "spool.sqlite"
    spool = RoadSpool(path, bloom_bytes=1024)
    try:
        with pytest.raises(network.RoadNetworkError, match="existing road spool"):
            RoadSpool(path)
        spool.add_footprint_node(network.Node(1, 0, 0))
        spool.flush_nodes()
        spool.add_touching_way(network.Way(10, (1, 2), (("highway", "primary"),)))
        spool.missing_filter()
        with pytest.raises(network.RoadNetworkError, match="required road node 2"):
            spool.finish()
    finally:
        spool.close()


def test_artifact_write_failure_preserves_previous_bytes(tmp_path, monkeypatch):
    output = tmp_path / "graph.json"
    output.write_bytes(b"previous qualified artifact")

    def broken(_):
        yield b"partial"
        raise network.RoadNetworkError("failed graph encoding")

    monkeypatch.setattr(network, "canonical_chunks", broken)
    with pytest.raises(network.RoadNetworkError, match="failed graph encoding"):
        network.write_graph(None, output)
    assert output.read_bytes() == b"previous qualified artifact"
    assert list(tmp_path.iterdir()) == [output]


def test_source_hash_refusal_precedes_graph_publication(tmp_path):
    road_pbf = pbf(tmp_path)
    extent = footprint(tmp_path)
    manifest = tmp_path / "manifest.json"
    manifest.write_text(
        json.dumps(
            {
                "pbf_sha256": "a" * 64,
                "pbf_bytes": road_pbf.stat().st_size,
                "pbf_url": "fixture",
                "replication_timestamp": "fixture",
                "footprint_sha256": extent.sha256,
            }
        )
    )
    with pytest.raises(network.RoadNetworkError, match="PBF SHA-256"):
        extraction._verified_source(road_pbf, manifest, extent, 0)


def test_cli_publishes_usable_graph_from_verified_pbf_and_authored_profile(tmp_path, capsys):
    road_pbf = pbf(tmp_path)
    footprint(tmp_path)
    extent_path = tmp_path / "footprint.json"
    extent_path.write_text(extent_path.read_text().replace("-84.999", "-84.996"))
    extent = Footprint(extent_path, 0)
    manifest = tmp_path / "manifest.json"
    manifest.write_text(
        json.dumps(
            {
                "pbf_sha256": hashlib.sha256(road_pbf.read_bytes()).hexdigest(),
                "pbf_bytes": road_pbf.stat().st_size,
                "pbf_url": "https://example.invalid/dated.osm.pbf",
                "replication_timestamp": "2026-09-08T20:21:01Z",
                "footprint_sha256": extent.sha256,
            }
        )
    )
    profile = tmp_path / "profile.json"
    profile.write_text(
        '{"gross_weight_kg":40000,"height_mm":4000,"width_mm":2550,"length_mm":16500,"default_maxheight_mm":4000,"evidence_class":"Designed"}'
    )
    output = tmp_path / "graph.json"
    assert (
        extraction.main(
            [
                "--pbf",
                str(road_pbf),
                "--source-manifest",
                str(manifest),
                "--footprint",
                str(extent_path),
                "--vehicle-profile",
                str(profile),
                "--spool",
                str(tmp_path / "spool.sqlite"),
                "--output",
                str(output),
                "--bloom-mib",
                "1",
            ]
        )
        == 0
    )
    artifact = json.loads(output.read_bytes())
    assert artifact["profile"]["gross_weight_kg"] == 40000
    assert artifact["source"]["pbf_sha256"] == hashlib.sha256(road_pbf.read_bytes()).hexdigest()
    assert {edge["way_id"] for edge in artifact["edges"]} == {10, 20}
    assert len(artifact["turn_rules"]) == 1
    assert artifact["turn_rules"][0]["kind"] == "only"
    assert artifact["diagnostics"] == []
    report = json.loads(capsys.readouterr().out.splitlines()[-1])
    assert report["sha256"] == hashlib.sha256(output.read_bytes()).hexdigest()


def test_invalid_extent_does_not_receive_silent_geometry_repair(tmp_path):
    path = tmp_path / "invalid.json"
    path.write_text('{"type":"Polygon","coordinates":[[[0,0],[1,1],[0,1],[1,0],[0,0]]]}')
    with pytest.raises(network.RoadNetworkError, match="valid, nonempty"):
        Footprint(path, 0)


def test_bloom_keeps_every_inserted_id_with_fixed_allocation():
    bloom = IdBloom(32)
    for ident in range(1, 10000):
        bloom.add(ident)
    assert len(bloom.bits) == 32
    assert all(ident in bloom for ident in range(1, 10000))


def test_native_bounds_strictly_enclose_buffered_footprint(tmp_path):
    extent = footprint(tmp_path)
    buffered = Footprint(tmp_path / "footprint.json", 100000)
    bounds = extraction.native_bounds_e7(buffered)
    assert bounds[0] / 1e7 < buffered.bounds[0]
    assert bounds[1] / 1e7 < buffered.bounds[1]
    assert bounds[2] / 1e7 > buffered.bounds[2]
    assert bounds[3] / 1e7 > buffered.bounds[3]
    assert bounds[0] < extent.bounds[0] * 1e7


@pytest.fixture
def native_osmium():
    import os
    import shutil
    from pathlib import Path

    executable = os.environ.get("BABYLON_OSMIUM_TOOL") or shutil.which("osmium")
    if not executable:
        pytest.skip("native preprocessing fixture requires pinned osmium-tool 1.18.0")
    binary = Path(executable)
    return binary, hashlib.sha256(binary.read_bytes()).hexdigest()


def source_manifest(tmp_path, original, extent):
    path = tmp_path / "source-manifest.json"
    path.write_text(
        json.dumps(
            {
                "pbf_sha256": hashlib.sha256(original.read_bytes()).hexdigest(),
                "pbf_bytes": original.stat().st_size,
                "pbf_url": "https://example.invalid/dated.osm.pbf",
                "replication_timestamp": "2026-09-08T20:21:01Z",
                "footprint_sha256": extent.sha256,
            }
        )
    )
    return path


def via_restrictions_pbf(tmp_path):
    xml = tmp_path / "source.osm"
    xml.write_text("""<osm version="0.6">
      <node id="1" version="1" lat="44" lon="-85"/>
      <node id="2" version="1" lat="44" lon="-84.9995"/>
      <node id="3" version="1" lat="44" lon="-84.9992"/>
      <node id="4" version="1" lat="44" lon="-84.9985"/>
      <node id="5" version="1" lat="44.0005" lon="-84.9995"/>
      <node id="6" version="1" lat="44" lon="-84.9993"><tag k="barrier" v="bollard"/></node>
      <node id="7" version="1" lat="44.0005" lon="-84.9993"/>
      <way id="10" version="1"><nd ref="1"/><nd ref="2"/><tag k="highway" v="primary"/></way>
      <way id="20" version="1"><nd ref="2"/><nd ref="3"/><tag k="highway" v="primary"/><tag k="bridge" v="yes"/><tag k="layer" v="1"/></way>
      <way id="30" version="1"><nd ref="3"/><nd ref="4"/><tag k="highway" v="primary"/></way>
      <way id="40" version="1"><nd ref="2"/><nd ref="5"/><tag k="highway" v="primary"/></way>
      <way id="50" version="1"><nd ref="6"/><nd ref="7"/><tag k="highway" v="footway"/></way>
      <relation id="100" version="1"><member type="way" ref="10" role="from"/><member type="node" ref="2" role="via"/><member type="way" ref="40" role="to"/><tag k="type" v="restriction"/><tag k="restriction" v="no_right_turn"/></relation>
      <relation id="110" version="1"><member type="way" ref="10" role="from"/><member type="way" ref="20" role="via"/><member type="way" ref="30" role="to"/><tag k="type" v="restriction:hgv"/><tag k="restriction:hgv" v="only_straight_on"/></relation>
    </osm>""")
    output = tmp_path / "source.osm.pbf"
    header = osmium.io.Header()
    header.set("osmosis_replication_timestamp", "2026-09-08T20:21:01Z")
    with osmium.SimpleWriter(str(output), header=header) as writer:
        for row in osmium.FileProcessor(str(xml)):
            writer.add(row)
    return output


def extract_fixture_graph(path, extent, database, source):
    spool = RoadSpool(database, bloom_bytes=1024)
    try:
        extract_pbf(path, extent, spool, progress=lambda _: None)
        return network.build_graph(
            spool.nodes(),
            spool.ways(),
            spool.restrictions(),
            source=source,
            profile=network.VehicleProfile(40000, 4000, 2550, 16500, 4000),
        ), tuple(spool.restrictions())
    finally:
        spool.close()


@pytest.mark.parametrize(
    "fixture_builder", [pbf, via_restrictions_pbf], ids=["boundary", "via-node-and-way"]
)
def test_native_preprocessing_preserves_actual_graph_and_original_authority(
    tmp_path, native_osmium, fixture_builder, monkeypatch
):
    binary, binary_sha256 = native_osmium
    original, extent = fixture_builder(tmp_path), footprint(tmp_path)
    manifest = source_manifest(tmp_path, original, extent)
    source = extraction._verified_source(original, manifest, extent, 0)
    regional, prepared = tmp_path / "regional.pbf", tmp_path / "roads.pbf"
    sidecar = tmp_path / "preparation.json"
    digest = extraction.prepare_native_pbf(
        source=source,
        footprint=extent,
        original=original,
        regional=regional,
        prepared=prepared,
        binary=binary,
        binary_sha256=binary_sha256,
        manifest=sidecar,
    )
    direct, direct_relations = extract_fixture_graph(
        original, extent, tmp_path / "direct.sqlite", source
    )
    accelerated, accelerated_relations = extract_fixture_graph(
        prepared, extent, tmp_path / "accelerated.sqlite", source
    )
    assert accelerated == direct
    assert accelerated_relations == direct_relations
    if fixture_builder is pbf:
        assert not accelerated.edges
        assert any(row.code == "unsupported_restriction" for row in accelerated.diagnostics)
        # Whole way and untouched relation members survive even when the other way
        # lies outside the extraction extent. The compiler still refuses that turn.
        assert accelerated_relations[0].members[-1].ref == 20
    else:
        assert accelerated.edges
        assert {rule.relation_id for rule in accelerated.turn_rules} == {100, 110}
        assert any(len(rule.prefix_edge_ids) >= 2 for rule in accelerated.turn_rules)
        assert any(dict(edge.tags).get("bridge") == "yes" for edge in accelerated.edges)
    recorded = json.loads(sidecar.read_bytes())
    assert recorded["source"] == extraction.asdict(source)
    assert recorded["native"]["sha256"] == binary_sha256
    assert recorded["prepared"]["sha256"] == hashlib.sha256(prepared.read_bytes()).hexdigest()
    assert recorded["regional"]["sha256"] == hashlib.sha256(regional.read_bytes()).hexdigest()
    assert all("--verbose" in argv for argv in recorded["commands"])
    original.unlink()  # Prepared verification must not reread the continent source.
    bound_source = extraction.verified_preparation_source(
        original=original,
        prepared=prepared,
        manifest=sidecar,
        source_manifest=manifest,
        footprint=extent,
    )
    assert bound_source.pbf_sha256 == source.pbf_sha256
    assert bound_source.extraction_version.endswith(f"native-preparation-sha256={digest}")
    profile = tmp_path / "profile.json"
    profile.write_text(
        '{"gross_weight_kg":40000,"height_mm":4000,"width_mm":2550,"length_mm":16500,"default_maxheight_mm":4000}'
    )
    output = tmp_path / "cli-graph.json"
    assert (
        extraction.main(
            [
                "--pbf",
                str(original),
                "--source-manifest",
                str(manifest),
                "--footprint",
                str(tmp_path / "footprint.json"),
                "--vehicle-profile",
                str(profile),
                "--spool",
                str(tmp_path / "cli.sqlite"),
                "--output",
                str(output),
                "--bloom-mib",
                "1",
                "--prepared-pbf",
                str(prepared),
                "--preparation-manifest",
                str(sidecar),
            ]
        )
        == 0
    )
    artifact = json.loads(output.read_bytes())
    assert artifact["source"] == extraction.asdict(bound_source)
    assert artifact["edges"] == json.loads(b"".join(network.canonical_chunks(direct)))["edges"]
    prepared.write_bytes(prepared.read_bytes() + b"changed")
    with pytest.raises(network.RoadNetworkError, match="prepared PBF path, SHA-256 or byte count"):
        extraction.verified_preparation_source(
            original=original,
            prepared=prepared,
            manifest=sidecar,
            source_manifest=manifest,
            footprint=extent,
        )


def test_completed_native_stage_record_keeps_actual_relative_argv(tmp_path, native_osmium):
    import subprocess

    binary, binary_sha256 = native_osmium
    original, extent = pbf(tmp_path), footprint(tmp_path)
    manifest = source_manifest(tmp_path, original, extent)
    source = extraction._verified_source(original, manifest, extent, 0)
    regional, prepared = tmp_path / "regional.pbf", tmp_path / "roads.pbf"
    commands = [
        list(argv)
        for argv in extraction.native_commands(binary, original, regional, prepared, extent)
    ]
    # Equivalent invocation spellings must be retained, rather than rewritten as
    # a fabricated launch record when recording already completed operator work.
    commands[0][-2] = original.name
    commands[0][-1:-1] = ["--output", regional.name]
    commands[0] = [arg for arg in commands[0] if not arg.startswith("--output=")]
    commands[1][-5] = f"--output={prepared.name}"
    commands[1][-4] = regional.name
    for command in commands:
        subprocess.run(command, cwd=tmp_path, check=True)
    sidecar = tmp_path / "preparation.json"
    extraction.record_native_preparation(
        source=source,
        footprint=extent,
        original=original,
        regional=regional,
        prepared=prepared,
        binary=binary,
        binary_sha256=binary_sha256,
        commands=commands,
        manifest=sidecar,
        working_directory=tmp_path,
    )
    assert json.loads(sidecar.read_bytes())["commands"] == commands
    extraction.verified_preparation_source(
        original=original,
        prepared=prepared,
        manifest=sidecar,
        source_manifest=manifest,
        footprint=extent,
    )
    unsafe = [*commands[1], "--omit-referenced"]
    with pytest.raises(network.RoadNetworkError, match="flags differ"):
        extraction.record_native_preparation(
            source=source,
            footprint=extent,
            original=original,
            regional=regional,
            prepared=prepared,
            binary=binary,
            binary_sha256=binary_sha256,
            commands=[commands[0], unsafe],
            manifest=tmp_path / "unsafe.json",
            working_directory=tmp_path,
        )
    assert not (tmp_path / "unsafe.json").exists()


def test_native_binary_pin_failure_cannot_publish_a_preparation(tmp_path, native_osmium):
    binary, _ = native_osmium
    original, extent = pbf(tmp_path), footprint(tmp_path)
    source = extraction._verified_source(
        original, source_manifest(tmp_path, original, extent), extent, 0
    )
    with pytest.raises(network.RoadNetworkError, match="binary SHA-256"):
        extraction.prepare_native_pbf(
            source=source,
            footprint=extent,
            original=original,
            regional=tmp_path / "regional.pbf",
            prepared=tmp_path / "roads.pbf",
            binary=binary,
            binary_sha256="0" * 64,
            manifest=tmp_path / "preparation.json",
        )
    assert not (tmp_path / "regional.pbf").exists()
    assert not (tmp_path / "preparation.json").exists()


def test_vector_coverage_includes_outer_and_hole_boundaries_without_joining_islands(tmp_path):
    from shapely.geometry import Point, shape
    from shapely.prepared import prep

    geometry = {
        "type": "MultiPolygon",
        "coordinates": [
            [[[0, 0], [4, 0], [4, 4], [0, 4], [0, 0]], [[1, 1], [1, 3], [3, 3], [3, 1], [1, 1]]],
            [[[6, 0], [8, 0], [8, 2], [6, 2], [6, 0]]],
        ],
    }
    path = tmp_path / "holes-and-islands.geojson"
    path.write_text(json.dumps(geometry))
    points = [
        (0, 0),
        (40000000, 20000000),
        (20000000, 20000000),
        (10000000, 20000000),
        (10000000, 10000000),
        (50000000, 10000000),
        (70000000, 10000000),
        (80000000, 20000000),
        (80000001, 10000000),
        (-1, 20000000),
    ]
    nodes = [network.Node(index + 1, x, y) for index, (x, y) in enumerate(points)]
    extent = Footprint(path, 0)
    assert [node.id for node in extent.covered_nodes(nodes)] == [1, 2, 4, 5, 7, 8]
    assert list(extent.covered_nodes([])) == []
    reference = prep(shape(geometry).buffer(100000 / 1e7))
    buffered = Footprint(path, 100000)
    assert list(buffered.covered_nodes(nodes)) == [
        node for node in nodes if reference.covers(Point(node.lon_e7 / 1e7, node.lat_e7 / 1e7))
    ]


@pytest.mark.parametrize(
    "fixture_builder", [pbf, via_restrictions_pbf], ids=["boundary", "via-node-and-way"]
)
def test_bounded_vector_extraction_matches_scalar_fixture_graph(
    tmp_path, monkeypatch, fixture_builder
):
    from shapely.geometry import Point
    from shapely.prepared import prep

    original, extent = fixture_builder(tmp_path), footprint(tmp_path)
    source = extraction._verified_source(
        original, source_manifest(tmp_path, original, extent), extent, 0
    )
    monkeypatch.setattr(extraction, "BATCH_SIZE", 2)
    batch_sizes = []
    vector_covered = Footprint.covered_nodes

    def recorded(self, nodes):
        batch_sizes.append(len(nodes))
        return vector_covered(self, nodes)

    monkeypatch.setattr(Footprint, "covered_nodes", recorded)
    actual, relations = extract_fixture_graph(original, extent, tmp_path / "vector.sqlite", source)
    assert batch_sizes and max(batch_sizes) == 2
    assert batch_sizes[-1] == 1  # Final partial input batch must be flushed.

    def scalar_reference(self, nodes):
        reference = prep(self.geometry)
        return (
            node for node in nodes if reference.covers(Point(node.lon_e7 / 1e7, node.lat_e7 / 1e7))
        )

    monkeypatch.setattr(Footprint, "covered_nodes", scalar_reference)
    expected, expected_relations = extract_fixture_graph(
        original, extent, tmp_path / "scalar.sqlite", source
    )
    assert actual == expected
    assert relations == expected_relations
