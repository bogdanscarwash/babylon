# Build-time transformation for the digest-pinned upstream postgres entrypoint.
# Every anchor must occur exactly once and in the expected order. Any upstream
# shape drift aborts the image build before the original entrypoint is replaced.

BEGIN {
    setup_anchor = "\t\tdocker_setup_env"
    create_directories_anchor = "\t\tdocker_create_db_directories"
    gosu_anchor = "\t\t\texec gosu postgres \"$BASH_SOURCE\" \"$@\""
    init_branch_anchor = "\t\tif [ -z \"$DATABASE_ALREADY_EXISTS\" ]; then"
    init_files_anchor = "\t\t\tdocker_process_init_files /docker-entrypoint-initdb.d/*"
    temp_stop_anchor = "\t\t\tdocker_temp_server_stop"
    unset_password_anchor = "\t\t\tunset PGPASSWORD"
}

{
    if ($0 == gosu_anchor) {
        print "\t\t\texec su-exec postgres \"$BASH_SOURCE\" \"$@\""
        gosu_count++
        gosu_line = NR
        next
    }

    print

    if ($0 == setup_anchor) {
        setup_count++
        setup_line = NR
        print "\t\tlocal babylon_lineage_marker=\"$PGDATA/.babylon-postgres-lineage-v1\""
        print "\t\tlocal babylon_lineage=\"babylon-postgres-lineage-v1|postgres=17|locale-provider=builtin|locale=C.UTF-8|encoding=UTF8|postgis=3.5.7|h3=4.5.0|h3_postgis=4.5.0|vector=0.8.5\""
        print "\t\tlocal babylon_lineage_state=\"empty\""
        print "\t\tlocal babylon_first_entry"
        print "\t\tif [ -e \"$PGDATA\" ] && [ ! -d \"$PGDATA\" ]; then"
        print "\t\t\tbabylon_lineage_state=\"refuse\""
        print "\t\telif [ -e \"$babylon_lineage_marker\" ]; then"
        print "\t\t\tif [ -f \"$babylon_lineage_marker\" ] \\"
        print "\t\t\t\t&& [ ! -L \"$babylon_lineage_marker\" ] \\"
        print "\t\t\t\t&& [ \"$(stat -c %a \"$babylon_lineage_marker\")\" = \"444\" ] \\"
        print "\t\t\t\t&& [ \"$(cat \"$babylon_lineage_marker\")\" = \"$babylon_lineage\" ] \\"
        print "\t\t\t\t&& [ \"$(wc -c < \"$babylon_lineage_marker\")\" -eq \"$((${#babylon_lineage} + 1))\" ] \\"
        print "\t\t\t\t&& [ -f \"$PGDATA/PG_VERSION\" ] \\"
        print "\t\t\t\t&& [ \"$(cat \"$PGDATA/PG_VERSION\")\" = \"17\" ] \\"
        print "\t\t\t\t&& [ \"$(wc -c < \"$PGDATA/PG_VERSION\")\" -eq 3 ]; then"
        print "\t\t\t\tbabylon_lineage_state=\"current\""
        print "\t\t\telse"
        print "\t\t\t\tbabylon_lineage_state=\"refuse\""
        print "\t\t\tfi"
        print "\t\telif [ -d \"$PGDATA\" ]; then"
        print "\t\t\tif ! babylon_first_entry=\"$(find \"$PGDATA\" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)\"; then"
        print "\t\t\t\tbabylon_lineage_state=\"refuse\""
        print "\t\t\telif [ -n \"$babylon_first_entry\" ]; then"
        print "\t\t\t\tbabylon_lineage_state=\"refuse\""
        print "\t\t\tfi"
        print "\t\tfi"
        print "\t\tif [ \"$babylon_lineage_state\" = \"refuse\" ]; then"
        print "\t\t\tprintf >&2 \"Error: refusing unrecognized PostgreSQL data directory at %s.\\n\" \"$PGDATA\""
        print "\t\t\tprintf >&2 \"This image accepts only an empty directory or the exact %s marker.\\n\" \"$babylon_lineage_marker\""
        print "\t\t\tprintf >&2 \"No PGDATA ownership, mode, or content mutation was attempted.\\n\""
        print "\t\t\tprintf >&2 \"Keep the old volume unchanged; use its original image for a separately planned offline logical dump/restore into babylon-pg-alpine-c-utf8-v1.\\n\""
        print "\t\t\texit 1"
        print "\t\tfi"
    }

    if ($0 == create_directories_anchor) {
        create_directories_count++
        create_directories_line = NR
    }

    if ($0 == init_branch_anchor) {
        init_branch_count++
        init_branch_line = NR
        print "\t\t\tPOSTGRES_INITDB_ARGS=\"${POSTGRES_INITDB_ARGS:+$POSTGRES_INITDB_ARGS }--locale-provider=builtin --builtin-locale=C.UTF-8 --encoding=UTF8\""
    }

    if ($0 == init_files_anchor) {
        init_files_count++
        init_files_line = NR
    }

    if ($0 == temp_stop_anchor) {
        temp_stop_count++
        temp_stop_line = NR
    }

    if ($0 == unset_password_anchor) {
        unset_password_count++
        unset_password_line = NR
        print "\t\t\tlocal babylon_lineage_tmp=\"${babylon_lineage_marker}.tmp.$$\""
        print "\t\t\ttest ! -e \"$babylon_lineage_marker\""
        print "\t\t\ttest ! -e \"$babylon_lineage_tmp\""
        print "\t\t\tprintf \"%s\\n\" \"$babylon_lineage\" > \"$babylon_lineage_tmp\""
        print "\t\t\tchmod 0444 \"$babylon_lineage_tmp\""
        print "\t\t\tmv \"$babylon_lineage_tmp\" \"$babylon_lineage_marker\""
    }
}

END {
    valid_counts = setup_count == 1 \
        && create_directories_count == 1 \
        && gosu_count == 1 \
        && init_branch_count == 1 \
        && init_files_count == 1 \
        && temp_stop_count == 1 \
        && unset_password_count == 1
    valid_order = setup_line < create_directories_line \
        && create_directories_line < gosu_line \
        && gosu_line < init_branch_line \
        && init_branch_line < init_files_line \
        && init_files_line < temp_stop_line \
        && temp_stop_line < unset_password_line
    if (!valid_counts || !valid_order) {
        print "entrypoint patch: refusing unexpected upstream entrypoint shape" > "/dev/stderr"
        exit 42
    }
}
