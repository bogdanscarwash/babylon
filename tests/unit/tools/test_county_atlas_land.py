"""Michigan atlas land behavior, using generated sources instead of a data drive."""

from __future__ import annotations

import copy
import hashlib
import sys
import zipfile
from pathlib import Path

import geopandas as gpd
import pytest
from pyproj import Transformer
from shapely.geometry import MultiPolygon, Point, Polygon, box

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))

import build_county_atlas as atlas  # type: ignore[import-not-found]  # noqa: E402
import make_county_place_h3_overlap_artifacts as source  # type: ignore[import-not-found]  # noqa: E402

TO_METRES = Transformer.from_crs(4269, 5070, always_xy=True)


def _water_archive(tmp_path: Path, coast: float = -84.5) -> dict[str, str]:
    """Write a tiny real shapefile ZIP, including an island and an inland lake."""
    island = box(-84.3, 44.3, -84.2, 44.4)
    xs = sorted({coast, -84.5, -84.0})
    shore = [(x, 44.0) for x in xs] + [(x, 45.0) for x in reversed(xs)]
    lake = Polygon(shore, [island.exterior.coords])
    water = [lake, box(-84.9, 44.7, -84.7, 44.8)]
    frame = gpd.GeoDataFrame(
        [
            {
                "ANSICODE": "00000001",
                "HYDROID": str(index + 1),
                "FULLNAME": "Generated water",
                "MTFCC": "H2030",
                "ALAND": 0,
                "AWATER": 1,
                "INTPTLAT": "+44.5000000",
                "INTPTLON": "-084.5000000",
                "geometry": geometry,
            }
            for index, geometry in enumerate(water)
        ],
        geometry="geometry",
        crs="EPSG:4269",
    )
    stem = "tl_2023_26001_areawater"
    members = tmp_path / "members"
    members.mkdir(parents=True)
    frame.to_file(members / f"{stem}.shp", engine="pyogrio", index=False)
    for suffix in ("shp.ea.iso.xml", "shp.iso.xml"):
        (members / f"{stem}.{suffix}").write_text("<generated-fixture/>", encoding="utf-8")
    destination = f"tiger/areawater/{stem}.zip"
    archive = tmp_path / destination
    archive.parent.mkdir(parents=True)
    with zipfile.ZipFile(archive, "w") as output:
        for member in sorted(members.iterdir()):
            output.writestr(zipfile.ZipInfo(member.name), member.read_bytes())
    return {"dest": destination, "sha256": hashlib.sha256(archive.read_bytes()).hexdigest()}


def _verified_fixture_land(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, coast: float = -84.5
) -> dict:
    pin = _water_archive(tmp_path, coast)
    monkeypatch.setattr(source, "TROVE", tmp_path)
    # Shared boundary nodes avoid artificial projection-chord slivers. The
    # county source is identical for both changed-water scenarios.
    xs = (-85.0, -84.6, -84.5, -84.0)
    county = Polygon([(x, 44.0) for x in xs] + [(x, 45.0) for x in reversed(xs)])
    observed = source.CountySource(1, {"26001": county}, county.bounds)
    return source.load_county_land_geometries(observed, {"26001": pin})


def _rows() -> list:
    return sorted(
        [
            atlas.CountyRow("01001", "Unchanged west", "AL", 1, box(-110, 30, -109, 31).wkt),
            atlas.CountyRow("01021", "Unchanged east", "AL", 1, box(-76, 46, -75, 47).wkt),
            atlas.CountyRow("02013", "Inset", "AK", 1, box(-151, 60, -150, 61).wkt),
            atlas.CountyRow("15001", "Inset", "HI", 1, box(-156, 19, -155, 20).wkt),
            atlas.CountyRow("26001", "Same legal name", "MI", 1, box(-85, 44, -84, 45).wkt),
            atlas.CountyRow("72001", "Inset", "PR", 1, box(-67, 18, -66, 19).wkt),
        ],
        key=lambda row: row.fips,
    )


def _grid_geometry(county: atlas.ProjectedCounty) -> MultiPolygon:
    polygons = []
    exterior = []
    holes = []
    for ring, is_hole in county.grid_rings:
        if is_hole:
            holes.append(ring)
        else:
            if exterior:
                polygons.append(Polygon(exterior, holes))
            exterior, holes = ring, []
    if exterior:
        polygons.append(Polygon(exterior, holes))
    return MultiPolygon(polygons)


def _grid_point(longitude: float, latitude: float, origin: tuple) -> Point:
    x, y = TO_METRES.transform(longitude, latitude)
    return Point((x - origin[0]) / origin[2], (y - origin[1]) / origin[2])


def test_build_clips_coast_preserves_island_and_hole_after_quantization(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Exercise the actual build/encoder; the old builder draws the lake as land."""
    land = _verified_fixture_land(tmp_path, monkeypatch)
    rows = _rows()
    captured = []
    encode = atlas.encode

    def capture(counties, origin, csr, report):
        captured.append((copy.deepcopy(counties), origin, csr))
        return encode(counties, origin, csr, report)

    monkeypatch.setattr(atlas, "read_counties", lambda _path, _report: rows)
    # raising=False makes the same behavior test run against the old builder:
    # it ignores this admitted land and fails the open-water assertion below.
    monkeypatch.setattr(atlas, "_read_michigan_land", lambda _report: land, raising=False)
    monkeypatch.setattr(atlas, "encode", capture)
    payload, report = atlas.build(tmp_path)
    again, _ = atlas.build(tmp_path)
    assert payload == again
    assert report.county_count == len(rows)
    assert report.byte_size < atlas.MAX_ARTIFACT_BYTES
    counties, origin, csr = captured[0]
    county = next(item for item in counties if item.fips == "26001")
    shape = _grid_geometry(county)
    assert not shape.contains(_grid_point(-84.1, 44.5, origin)), "open lake became county land"
    assert not shape.contains(_grid_point(-84.8, 44.75, origin)), "inland water hole was filled"
    assert shape.contains(_grid_point(-84.25, 44.35, origin)), "land island was dropped"
    assert shape.contains(_grid_point(-84.8, 44.4, origin)), "mainland was removed"
    assert shape.contains(Point(county.grid_centroid)), "display anchor must remain on land"
    assert (county.fips, county.name, county.state_abbrev) == ("26001", "Same legal name", "MI")
    assert county.area_sq_km == pytest.approx(land["26001"].area / 1e6)

    # The old preparation path for all other counties, including insets, stays exact.
    unchanged = atlas.project_counties(rows)
    atlas.place_insets(unchanged, atlas.Report())
    atlas.quantize(unchanged, atlas.Report())
    for before, after in zip(unchanged, counties, strict=True):
        if before.fips != "26001":
            assert before == after
    assert csr == atlas.build_csr(unchanged, atlas.Report())


def test_changed_verified_water_changes_land_without_changing_identity(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    first = _verified_fixture_land(tmp_path / "first", monkeypatch)
    second = _verified_fixture_land(tmp_path / "second", monkeypatch, coast=-84.6)
    before = atlas.project_counties(_rows())
    after = copy.deepcopy(before)
    atlas._apply_michigan_land(before, first)
    atlas._apply_michigan_land(after, second)
    assert [county.fips for county in before] == [county.fips for county in after]
    assert [county.name for county in before] == [county.name for county in after]
    first_mi = next(county for county in before if county.fips == "26001")
    second_mi = next(county for county in after if county.fips == "26001")
    assert second_mi.rings != first_mi.rings
    assert second_mi.area_sq_km < first_mi.area_sq_km
    assert atlas.build_csr(before, atlas.Report()) == atlas.build_csr(after, atlas.Report())


def test_water_hash_is_checked_before_decode(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    pin = _water_archive(tmp_path)
    (tmp_path / pin["dest"]).write_bytes(b"changed source")
    monkeypatch.setattr(source, "TROVE", tmp_path)

    def refuse_decode(*_args, **_kwargs):
        pytest.fail("unverified water reached the geometry decoder")

    monkeypatch.setattr(gpd, "read_file", refuse_decode)
    county = box(-85, 44, -84, 45)
    with pytest.raises(source.OverlapBuildError, match="source_sha256"):
        source.load_county_land_geometries(
            source.CountySource(1, {"26001": county}, county.bounds), {"26001": pin}
        )


@pytest.mark.parametrize("missing", ["26001", "26999"])
def test_land_replacement_refuses_identity_mismatch(missing: str) -> None:
    counties = atlas.project_counties(_rows())
    land = {} if missing == "26001" else {missing: box(0, 0, 1, 1)}
    before = copy.deepcopy(counties)
    with pytest.raises(ValueError, match="county identities disagree"):
        atlas._apply_michigan_land(counties, land)
    assert counties == before


def test_anchor_uses_final_rings_instead_of_centroid_in_water() -> None:
    county = atlas.ProjectedCounty("26001", "Land", "MI", 1, "conus")
    county.rings = [
        ([(0, 0), (100, 0), (100, 100), (0, 100)], False),
        ([(20, 20), (80, 20), (80, 80), (20, 80)], True),
    ]
    county.centroid = (50, 50)
    atlas.quantize([county], atlas.Report())
    geometry = _grid_geometry(county)
    assert not geometry.contains(Point(32768, 32768))
    assert geometry.contains(Point(county.grid_centroid))


def test_grid_preserves_land_and_water_when_subgrid_inlets_and_islands_collapse() -> None:
    # A metre per grid unit makes the narrow inlet and hole spur round onto
    # themselves, as in Alcona. A third island becomes three collinear points.
    mainland = Polygon(
        [(0, 0), (100, 0), (100, 100), (50.4, 100), (50.4, 40), (50.2, 40), (50.2, 100), (0, 100)],
        [[(10, 10), (30, 10), (30, 20.2), (40, 20.2), (40, 20.4), (30, 20.4), (30, 30), (10, 30)]],
    )
    land = MultiPolygon(
        [mainland, box(120, 0, 140, 20), Polygon([(150, 0.1), (151, 0.3), (152, 0.1)])]
    )
    assert land.is_valid
    county = atlas.ProjectedCounty("26001", "Land", "MI", 1, "conus")
    county.rings = atlas._rings_of(land)
    extent = atlas.ProjectedCounty("01001", "Grid extent", "AL", 1, "conus")
    extent.rings = atlas._rings_of(box(0, 0, atlas.GRID_MAX, atlas.GRID_MAX))
    report = atlas.Report()
    origin = atlas.quantize([county, extent], report)
    assert origin == (0, 0, 1)
    shape = _grid_geometry(county)
    assert shape.is_valid
    assert shape.contains(Point(70, 50)), "mainland must survive"
    assert shape.contains(Point(130, 10)), "resolvable island must survive"
    assert not shape.covers(Point(151, 0)), "collinear island must not become land"
    assert not shape.contains(Point(20, 20)), "resolvable water must survive"
    assert shape.contains(Point(county.grid_centroid))
    before = copy.deepcopy(county.grid_rings)
    atlas.quantize([county, extent], atlas.Report())
    assert county.grid_rings == before
    assert report.land_precision_changes[0][:3] == ("26001", 4, 3)
    assert 0 < report.land_precision_changes[0][3] < 0.01


def test_grid_refuses_invalid_source_instead_of_repairing_it() -> None:
    county = atlas.ProjectedCounty("26001", "Land", "MI", 1, "conus")
    county.rings = [([(0, 0), (100, 100), (0, 100), (100, 0)], False)]
    with pytest.raises(ValueError, match="invalid land geometry"):
        atlas.quantize([county], atlas.Report())
    assert county.grid_rings == []


def test_anchor_rechecks_integer_rounding_and_disconnected_islands() -> None:
    # The largest polygon's representative point rounds onto its hole boundary.
    rings = [
        ([(0, 0), (5, 0), (5, 5), (0, 5)], False),
        ([(1, 1), (4, 1), (4, 4), (1, 4)], True),
        ([(10, 0), (14, 0), (14, 4), (10, 4)], False),
    ]
    anchor = atlas._land_grid_anchor("26001", rings)
    geometry = _grid_geometry(
        atlas.ProjectedCounty("26001", "Land", "MI", 1, "conus", grid_rings=rings)
    )
    assert geometry.contains(Point(anchor))
    assert anchor == (12, 2)
    assert anchor == atlas._land_grid_anchor("26001", rings)


def test_anchor_refuses_a_grid_with_no_interior_integer_point() -> None:
    with pytest.raises(ValueError, match="no representable interior land anchor"):
        atlas._land_grid_anchor("26001", [([(0, 0), (1, 0), (0, 1)], False)])


def test_admission_checks_county_source_before_decode(monkeypatch: pytest.MonkeyPatch) -> None:
    events = []
    monkeypatch.setattr(source, "verify_toolchain", lambda: events.append("toolchain"))
    monkeypatch.setattr(
        source, "county_source_pin", lambda: {"dest": "source.zip", "sha256": "a" * 64}
    )

    def changed_source(_path, _digest):
        events.append("verify")
        raise ValueError("changed county source")

    def refuse_decode(_path):
        pytest.fail("unverified county reached the decoder")

    monkeypatch.setattr(source, "verify_source_archive", changed_source)
    monkeypatch.setattr(source, "load_county_source", refuse_decode)
    report = atlas.Report()
    with pytest.raises(ValueError, match="changed county source"):
        atlas._read_michigan_land(report)
    assert events == ["toolchain", "verify"]
    assert report.inputs == []


@pytest.mark.parametrize("national, counties", [(3234, 83), (3235, 82), (3235, 84)])
def test_admission_requires_the_complete_checked_cohort(
    monkeypatch: pytest.MonkeyPatch, national: int, counties: int
) -> None:
    monkeypatch.setattr(source, "verify_toolchain", lambda: None)
    monkeypatch.setattr(
        source, "county_source_pin", lambda: {"dest": "source.zip", "sha256": "a" * 64}
    )
    monkeypatch.setattr(source, "verify_source_archive", lambda *_args: None)
    observed = source.CountySource(
        national, {f"26{index:03}": box(0, 0, 1, 1) for index in range(counties)}, (0, 0, 1, 1)
    )
    monkeypatch.setattr(source, "load_county_source", lambda _path: observed)
    with pytest.raises(ValueError, match="all 83 counties"):
        atlas._read_michigan_land(atlas.Report())


def test_report_names_mixed_presentation_vintages(capsys: pytest.CaptureFixture[str]) -> None:
    atlas.print_report(atlas.Report())
    output = capsys.readouterr().out
    assert "Michigan: Derived land from TIGER/Line 2023 COUNTY minus 2023 AREAWATER" in output
    assert "Other counties: TIGER/Line 2024 legal boundaries, without water subtraction" in output
    assert "Identity and adjacency: existing county dimensions and county_adjacency.json" in output
    assert "no mechanics change" in output
