add_rules("mode.debug", "mode.release")

target("server")

    set_kind("binary")

    add_files("src/*.rs")
    add_files("src/plugin/*.rs")
    add_files("src/service/*.rs")

    add_files("src/bin/server.rs")

target_end()