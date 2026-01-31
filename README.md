# Commander.rs - A DSL for Command Automation

A Rust-based domain-specific language (DSL) interpreter for defining and executing command workflows with support for dependencies, conditionals, loops, and variable substitution.

## Features

- **Hierarchical Commands**: Organize commands in nested structures
- **Variable Substitution**: Define and use variables with `$variable` syntax
- **Dependencies**: Declare command dependencies that execute automatically
- **Control Flow**: Support for `if/else` conditionals and `for` loops
- **Shell Execution**: Execute shell commands with `exec()`
- **Parameters**: Pass arguments to commands
- **Documentation**: Inline documentation with `///` comments

## Installation

```bash
cargo build --release
```

## Usage

Create a `Make.cmd` file with your command definitions, then run:

```bash
# Execute a top-level command
cmd <command>

# Execute a subcommand
cmd <command> <subcommand>

# Pass arguments (prefixed with --)
cmd <command> --arg1 --arg2
```

## Syntax

### Basic Command

```
build {
    exec(echo "Building project...")
    exec(cargo build --release)
}
```

### Commands with Parameters

```
deploy(environment) {
    exec(echo "Deploying to $environment")
    exec(./deploy.sh $environment)
}
```

Run with: `./make_cmd deploy --production`

### Variables

```
setup {
    let project_name = "my-app";
    let version = "1.0.0";
    exec(echo "Setting up $project_name v$version")
}
```

### Dependencies

```
test {
    depends(build)
    exec(cargo test)
}

build {
    exec(cargo build)
}
```

### Conditionals

```
deploy(env) {
    if $env == "production" {
        exec(echo "Deploying to production with extra checks")
        exec(./safety-check.sh)
    } else {
        exec(echo "Deploying to $env")
    }
    exec(./deploy.sh $env)
}
```

### Loops

```
clean {
    for dir in ["target", "dist", "build"] {
        exec(rm -rf $dir)
        exec(echo "Cleaned $dir")
    }
}
```

### Nested Commands (Subcommands)

```
docker {
    build {
        exec(docker build -t myapp .)
    }
    
    run {
        depends(build)
        exec(docker run -p 8080:8080 myapp)
    }
    
    clean {
        exec(docker system prune -f)
    }
}
```

Run with: `cmd docker build`

### Documentation Comments

```
/// Builds the entire project in release mode
build {
    exec(cargo build --release)
}
```

## Language Reference

### Statements

- **`let variable = value;`** - Variable assignment
- **`exec(command)`** - Execute shell command
- **`depends(cmd1, cmd2, ...)`** - Declare dependencies
- **`if condition { ... } else { ... }`** - Conditional execution
- **`for var in [items] { ... }`** - Loop over items

### Operators

- `==` - Equality
- `!=` - Inequality
- `>` - Greater than
- `<` - Less than

### Values

- **String literals**: `"hello world"`
- **Variables**: `$variable_name`

### Comments

- **Line comments**: `// comment` or `@REM comment`
- **Block comments**: `/* comment */`
- **Doc comments**: `/// documentation`

## Example: Complete Workflow

```
/// Main build pipeline
ci {
    depends(clean)
    
    let env = "development";
    
    build {
        exec(echo "Building in $env mode...")
        exec(cargo build)
    }
    
    test {
        depends(build)
        exec(cargo test)
    }
    
    lint {
        exec(cargo clippy)
    }
}

clean {
    for dir in ["target", "dist"] {
        exec(rm -rf $dir)
    }
}
```

Run the full CI pipeline:
```bash
cmd ci
```

Run only tests (with auto-build via dependency):
```bash
cmd ci test
```

## Output

The interpreter provides colored console output:
- 🔵 **[exec]** - Command execution
- 🟡 **[set]** - Variable assignment
- 🟣 **[depends]** - Dependency execution
- 🟢 **[param]** - Parameter binding
- 🔴 **[error]** - Error messages

## Error Handling

- Command failures are reported but don't stop execution
- Missing subcommands are reported with clear error messages
- Parse errors show the location of syntax issues

## License

This project uses the Pest parser library for grammar parsing.
