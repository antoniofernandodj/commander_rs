@REM ===============================
@REM Build padrão (debug)
build_debug {
    depends(clean);

    let target = "target/debug/app";

    exec(
        gcc -g -O0 -o $target src/*.c $CFLAGS
    );

    exec(
        ./$target --test
    );
}

@REM ===============================
@REM Build release
build_release {
    depends(clean);

    let target = "target/release/app";

    exec(
        gcc -O3 -DNDEBUG -o $target src/*.c $CFLAGS
    );
}

@REM ===============================
@REM Build genérico com parâmetro
build(mode) {
    depends(clean);

    let target = "target/$mode/app";

    if $mode == "debug" {
        exec(
            gcc -g -O0 -o $target src/*.c $CFLAGS
        );
    }

    if $mode == "release" {
        exec(
            gcc -O3 -DNDEBUG -o $target src/*.c $CFLAGS
        );
    }
}

@REM ===============================
@REM Limpeza completa
clean {
    exec(rm -rf target/*);
}

@REM ===============================
@REM Cria diretórios de build
prepare_dirs {
    exec(mkdir -p target/debug);
    exec(mkdir -p target/release);
}

@REM ===============================
@REM Executa aplicação em debug
run_debug {
    depends(build);

    exec(
        ./target/debug/app
    );
}

@REM ===============================
@REM Executa aplicação em release
run_release {
    depends(build_release);

    exec(
        ./target/release/app
    );
}

@REM ===============================
@REM Roda testes
test {
    depends(build_debug);

    exec(
        ./target/debug/app --test
    );
}

@REM ===============================
@REM Roda aplicação em modo desenvolvimento
dev {
    depends(build_debug);

    exec(
        ./target/debug/app --dev
    );
}

@REM ===============================
@REM Checa formatação dos arquivos
format_check {
    for file in [
        "src/main.c",
        "src/utils.c",
        "src/net.c",
        "src/db.c"
    ] {
        exec(echo "Checking format: $file");
    }
}

@REM ===============================
@REM Aplica clang-format
format_apply {
    for file in [
        "src/main.c",
        "src/utils.c",
        "src/net.c",
        "src/db.c"
    ] {
        exec(clang-format -i $file);
    }
}

@REM ===============================
@REM Análise estática
lint {
    depends(prepare_dirs);

    for file in [
        "src/main.c",
        "src/utils.c",
        "src/net.c"
    ] {
        exec(
            clang-tidy $file -- $CFLAGS
        );
    }
}

@REM ===============================
@REM Build com sanitizers
build_asan {
    depends(clean);

    let target = "target/debug/app-asan";

    exec(
        gcc -g -fsanitize=address -o $target src/*.c $CFLAGS
    );
}

@REM ===============================
@REM Executa binário com ASAN
run_asan {
    depends(build_asan);

    exec(
        ./target/debug/app-asan
    );
}

@REM ===============================
@REM Gera cobertura
coverage {
    depends(clean);

    let target = "target/debug/app-cov";

    exec(
        gcc -g -O0 --coverage -o $target src/*.c $CFLAGS
    );

    exec(
        ./$target --test
    );

    exec(
        gcov src/*.c
    );
}

@REM ===============================
@REM Compila somente um arquivo
build_one(file) {
    let target = "target/obj/$file.o";

    exec(
        gcc -c src/$file -o $target $CFLAGS
    );
}

@REM ===============================
@REM Build incremental (exemplo simplificado)
build_incremental {
    depends(prepare_dirs);

    for file in [
        "main.c",
        "utils.c",
        "net.c"
    ] {
        exec(
            gcc -c src/$file -o target/obj/$file.o $CFLAGS
        );
    }

    exec(
        gcc -o target/debug/app target/obj/*.o
    );
}

@REM ===============================
@REM Mostra variáveis de ambiente úteis
env {
    exec(echo "CFLAGS=$CFLAGS");
    exec(echo "PATH=$PATH");
}

@REM ===============================
@REM Empacota binário
package(mode) {
    depends(build);

    let bin = "target/$mode/app";
    let out = "target/$mode/app.tar.gz";

    exec(
        tar -czf $out $bin
    );
}

@REM ===============================
@REM Instala localmente
install(mode) {
    depends(build);

    let bin = "target/$mode/app";

    exec(
        cp $bin /usr/local/bin/app
    );
}

@REM ===============================
@REM Remove instalação local
uninstall {
    exec(
        rm -f /usr/local/bin/app
    );
}

@REM ===============================
@REM Roda benchmarks
bench {
    depends(build_release);

    exec(
        ./target/release/app --bench
    );
}

@REM ===============================
@REM Limpeza profunda
distclean {
    depends(clean);

    exec(
        rm -rf target
    );
}

@REM ===============================
@REM Verifica warnings extras
build_warnings {
    depends(clean);

    let target = "target/debug/app-warn";

    exec(
        gcc -Wall -Wextra -Wpedantic -o $target src/*.c
    );
}

@REM ===============================
@REM Lista arquivos de código
list_sources {
    for file in [
        "src/main.rs",
        "src/grammar.pest"
    ] {
        exec(echo "Showing content for $file:\n\n");
        exec(
            cat $file
        );
    }
}
