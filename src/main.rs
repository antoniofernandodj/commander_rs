use pest::Parser;
use pest_derive::Parser;
use pest::iterators::Pair;
use std::collections::HashMap;
use std::process::Command;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct DSLParser;

#[derive(Debug)]
pub struct CommandNode {
    pub name: String,
    pub params: Vec<String>,
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Command(CommandNode),
    Exec(String),
    Assignment(String, String),
    Depends(Vec<String>),
    If {
        condition: Condition,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    For {
        var: String,
        items: Vec<String>,
        block: Vec<Statement>,
    },
}

#[derive(Debug)]
pub struct Condition {
    pub left: String,
    pub op: String,
    pub right: String,
}

#[derive(Debug, Clone)]
pub struct Environment {
    vars: HashMap<String, String>,
}

impl Environment {
    fn new() -> Self {
        Self { vars: HashMap::new() }
    }
    
    fn set(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }
    
    fn expand(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (key, value) in &self.vars {
            result = result.replace(&format!("${}", key), value);
        }
        result
    }
    
    fn eval_condition(&self, cond: &Condition) -> bool {
        let left = self.expand(&cond.left);
        let right = self.expand(&cond.right);
        
        match cond.op.as_str() {
            "==" => left == right,
            "!=" => left != right,
            ">" => left > right,
            "<" => left < right,
            _ => false,
        }
    }
}

fn parse_value(pair: Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::string_lit => {
            pair.as_str().trim_matches('"').to_string()
        }
        Rule::variable => {
            pair.as_str().to_string()
        }
        _ => pair.as_str().to_string(),
    }
}

fn parse_statements(pairs: pest::iterators::Pairs<Rule>) -> Vec<Statement> {
    let mut statements = Vec::new();
    
    for stmt in pairs {
        match stmt.as_rule() {
            Rule::node => {
                statements.push(Statement::Command(parse_node(stmt)));
            }
            Rule::exec => {
                let body = stmt.into_inner().next().unwrap().as_str().to_string();
                statements.push(Statement::Exec(body));
            }
            Rule::assignment => {
                let mut parts = stmt.into_inner();
                let var_name = parts.next().unwrap().as_str().to_string();
                let value_pair = parts.next().unwrap();
                let value = parse_value(value_pair);
                statements.push(Statement::Assignment(var_name, value));
            }
            Rule::depends => {
                let deps: Vec<String> = stmt.into_inner()
                    .map(|p| p.as_str().to_string())
                    .collect();
                statements.push(Statement::Depends(deps));
            }
            Rule::if_stmt => {
                let mut parts = stmt.into_inner();
                
                let cond_pair = parts.next().unwrap();
                let mut cond_parts = cond_pair.into_inner();
                let left = parse_value(cond_parts.next().unwrap());
                let op = cond_parts.next().unwrap().as_str().to_string();
                let right = parse_value(cond_parts.next().unwrap());
                
                let condition = Condition { left, op, right };
                
                let then_block_pair = parts.next().unwrap();
                let then_block = parse_statements(then_block_pair.into_inner());
                
                let else_block = parts.next().map(|else_pair| {
                    parse_statements(else_pair.into_inner())
                });
                
                statements.push(Statement::If {
                    condition,
                    then_block,
                    else_block,
                });
            }
            Rule::for_stmt => {
                let mut parts = stmt.into_inner();
                let var = parts.next().unwrap().as_str().to_string();
                
                let array_pair = parts.next().unwrap();
                let items: Vec<String> = array_pair.into_inner()
                    .map(|p| parse_value(p))
                    .collect();
                
                let block_pair = parts.next().unwrap();
                let block = parse_statements(block_pair.into_inner());
                
                statements.push(Statement::For { var, items, block });
            }
            _ => {}
        }
    }
    
    statements
}

fn parse_node(pair: Pair<Rule>) -> CommandNode {
    let mut inner = pair.into_inner();
    
    let mut current = inner.next().unwrap();
    if current.as_rule() == Rule::doc_comment {
        current = inner.next().unwrap();
    }
    
    let name = current.as_str().to_string();
    
    current = inner.next().unwrap();
    
    let params = if current.as_rule() == Rule::param_list {
        let p: Vec<String> = current.into_inner()
            .map(|p| p.as_str().to_string())
            .collect();
        current = inner.next().unwrap();
        p
    } else {
        Vec::new()
    };
    
    let statements = parse_statements(current.into_inner());
    
    CommandNode { name, params, statements }
}

fn parse_program(input: &str) -> Vec<CommandNode> {
    let mut pairs = DSLParser::parse(Rule::program, input)
        .expect("parse error");
    
    let program = pairs.next().unwrap();
    
    program.into_inner()
        .filter(|p| p.as_rule() == Rule::node)
        .map(|p| parse_node(p))
        .collect()
}

fn execute_statements_only(
    statements: &[Statement], 
    path: &mut Vec<String>, 
    env: &mut Environment, 
    all_nodes: &HashMap<String, &CommandNode>
) {
    for stmt in statements {
        match stmt {
            Statement::Command(_) => {
                // Ignora sub-comandos
            }
            Statement::Exec(cmd) => {
                let expanded = env.expand(cmd);
                println!("\x1b[36m[exec]\x1b[0m {}", expanded.trim());
                
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&expanded)
                    .output();

                match output {
                    Ok(result) => {
                        if !result.stdout.is_empty() {
                            print!("{}", String::from_utf8_lossy(&result.stdout));
                        }
                        
                        if !result.status.success() {
                            eprintln!("\x1b[31m[error]\x1b[0m Command failed with status: {}", 
                                result.status);
                            if !result.stderr.is_empty() {
                                eprintln!("{}", String::from_utf8_lossy(&result.stderr));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\x1b[31m[error]\x1b[0m Failed to execute command: {}", e);
                    }
                }
            }
            Statement::Assignment(name, value) => {
                let expanded = env.expand(value);
                env.set(name.clone(), expanded.clone());
                println!("\x1b[33m[set]\x1b[0m {} = {}", name, expanded);
            }
            Statement::Depends(deps) => {
                for dep in deps {
                    if let Some(dep_node) = all_nodes.get(dep) {
                        println!("\x1b[35m[depends]\x1b[0m {}", dep);
                        let mut dep_path = Vec::new();
                        execute(dep_node, &mut dep_path, env, all_nodes, &[], None);
                    }
                }
            }
            Statement::If { condition, then_block, else_block } => {
                if env.eval_condition(condition) {
                    execute_statements_only(then_block, path, env, all_nodes);
                } else if let Some(else_stmts) = else_block {
                    execute_statements_only(else_stmts, path, env, all_nodes);
                }
            }
            Statement::For { var, items, block } => {
                for item in items {
                    let expanded = env.expand(item);
                    env.set(var.clone(), expanded);
                    execute_statements_only(block, path, env, all_nodes);
                }
            }
        }
    }
}

fn find_subnode<'a>(node: &'a CommandNode, name: &str) -> Option<&'a CommandNode> {
    for stmt in &node.statements {
        if let Statement::Command(child) = stmt {
            if child.name == name {
                return Some(child);
            }
        }
    }
    None
}

fn execute(
    node: &CommandNode, 
    path: &mut Vec<String>, 
    env: &mut Environment, 
    all_nodes: &HashMap<String, &CommandNode>,
    args: &[String],
    subpath: Option<&[String]>
) {
    path.push(node.name.clone());
    
    // Define parâmetros
    for (i, param) in node.params.iter().enumerate() {
        if let Some(arg) = args.get(i) {
            env.set(param.clone(), arg.clone());
            println!("\x1b[32m[param]\x1b[0m {} = {}", param, arg);
        }
    }

    // MUDANÇA: SEMPRE executa os statements do nó atual (exceto sub-comandos)
    // Isso garante que variáveis, depends, etc sejam processados
    execute_statements_only(&node.statements, path, env, all_nodes);

    // Se há um subpath, navega até o sub-nó
    if let Some(sub) = subpath {
        if !sub.is_empty() {
            if let Some(child) = find_subnode(node, &sub[0]) {
                // Recursivamente executa o filho com o resto do subpath
                execute(child, path, env, all_nodes, &[], Some(&sub[1..]));
            } else {
                eprintln!("\x1b[31m[error]\x1b[0m Subcommand '{}' not found in '{}'", sub[0], node.name);
            }
        }
        // Se subpath está vazio, já executamos os statements acima
    } else {
        // Sem subpath, executa todos os sub-comandos também
        for stmt in &node.statements {
            if let Statement::Command(child) = stmt {
                execute(child, path, env, all_nodes, &[], None);
            }
        }
    }

    path.pop();
}

fn find_node<'a>(
    nodes: &'a [CommandNode],
    name: &str,
) -> Option<&'a CommandNode> {
    nodes.iter().find(|n| n.name == name)
}

fn main() {
    let input = std::fs::read_to_string("Make.cmd")
        .expect("Não foi possível ler o arquivo");

    let nodes = parse_program(&input);
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() == 0 {
        eprintln!("\x1b[31m[error]\x1b[0m No command specified");
        return
    }

    let all_nodes: HashMap<String, &CommandNode> = nodes.iter()
        .map(|n| (n.name.clone(), n))
        .collect();

    let mut path = Vec::new();
    let mut env = Environment::new();

    if args.is_empty() {
        let start = nodes.first().expect("nenhum nó encontrado");
        execute(start, &mut path, &mut env, &all_nodes, &[], None);
    } else {
        let mut cmd_path = vec![];
        let mut cmd_args = vec![];

        for arg in &args {
            if arg.starts_with("--") {
                cmd_args.push(arg.trim_start_matches("--").to_string());
            } else {
                cmd_path.push(arg.clone());
            }
        }

        if cmd_path.is_empty() {
            eprintln!("\x1b[31m[error]\x1b[0m No command specified");
            return;
        }

        let root_name = &cmd_path[0];
        let subpath = &cmd_path[1..];

        if let Some(root_node) = find_node(&nodes, root_name) {
            if subpath.is_empty() {
                execute(root_node, &mut path, &mut env, &all_nodes, &cmd_args, None);
            } else {
                execute(root_node, &mut path, &mut env, &all_nodes, &cmd_args, Some(subpath));
            }
        } else {
            eprintln!("\x1b[31m[error]\x1b[0m Command '{}' not found", root_name);
        }
    }
}