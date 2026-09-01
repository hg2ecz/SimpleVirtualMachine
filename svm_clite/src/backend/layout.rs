use std::collections::{BTreeMap, HashMap};

pub const DATA_BASE: u16 = 0x8000;
const DATA_LIMIT: u32 = 0xFF00;

#[derive(Clone, Debug)]
pub struct Var {
    pub addr: u16,
    pub width: u16,
}

#[derive(Clone, Debug, Default)]
pub struct Fun {
    pub params: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub vars: HashMap<String, Var>,
    pub funcs: HashMap<String, Fun>,
    pub globals: BTreeMap<String, (String, Option<u16>)>,
    next: u32,
}

pub fn width_of(ty: &str) -> u16 {
    if matches!(ty, "bool" | "u8" | "i8") {
        1
    } else {
        2
    }
}

fn qualified(function: &str, name: &str) -> String {
    format!("{function}::{name}")
}

pub fn parse_fn_header(line: &str) -> Result<(String, Vec<(String, String)>), String> {
    let rest = line.strip_prefix("fn ").ok_or("bad function header")?;
    let left = rest.find('(').ok_or("bad function header")?;
    let right = rest.rfind(')').ok_or("bad function header")?;
    let name = rest[..left].trim().to_owned();
    rest[right + 1..]
        .trim()
        .strip_prefix("->")
        .ok_or("bad function return")?;

    let mut params = Vec::new();
    let inside = &rest[left + 1..right];
    if !inside.trim().is_empty() {
        for param in inside.split(',') {
            let mut parts = param.split_whitespace();
            let ty = parts.next().ok_or("bad parameter")?.to_owned();
            let name = parts.next().ok_or("bad parameter")?.to_owned();
            params.push((name, ty));
        }
    }
    Ok((name, params))
}

fn parse_decl(rest: &str, kind: &str) -> Result<(String, String, u16), String> {
    let before_init = rest.split_once(" = ").map_or(rest, |(left, _)| left);
    let mut parts = before_init.split_whitespace();
    let ty = parts
        .next()
        .ok_or_else(|| format!("bad {kind}"))?
        .to_owned();
    let mut name = parts
        .next()
        .ok_or_else(|| format!("bad {kind}"))?
        .to_owned();
    let mut count = 1u16;
    if let Some(left) = name.find('[') {
        let right = name.find(']').ok_or_else(|| format!("bad {kind} array"))?;
        count = name[left + 1..right]
            .parse()
            .map_err(|_| format!("bad {kind} array length"))?;
        name.truncate(left);
    }
    Ok((ty, name, count))
}

fn allocate(next: &mut u32, width: u16) -> Result<u16, String> {
    let addr = *next;
    if addr + u32::from(width) > DATA_LIMIT {
        return Err("C-Lite static data exceeds RAM below MMIO".into());
    }
    *next += u32::from(width);
    Ok(addr as u16)
}

impl Layout {
    pub fn scan(lines: &[&str], allocate_temps: bool) -> Result<Self, String> {
        let mut vars = HashMap::new();
        let mut funcs = HashMap::new();
        let mut globals = BTreeMap::new();
        let mut next = u32::from(DATA_BASE);
        let mut current = String::new();

        for line in lines {
            if let Some(rest) = line.strip_prefix("global ") {
                let (ty, name, count) = parse_decl(rest, "global")?;
                let init = rest
                    .split_once(" = ")
                    .map(|(_, value)| {
                        value
                            .trim()
                            .parse::<u16>()
                            .map_err(|_| "bad global initializer")
                    })
                    .transpose()?;
                let width = width_of(&ty);
                let addr = allocate(&mut next, width.saturating_mul(count))?;
                vars.insert(name.clone(), Var { addr, width });
                globals.insert(name, (ty, init));
                continue;
            }

            if line.starts_with("fn ") {
                let (name, params) = parse_fn_header(line)?;
                current = name.clone();
                let mut function = Fun::default();
                for (param, ty) in params {
                    let width = width_of(&ty);
                    let addr = allocate(&mut next, width)?;
                    vars.insert(qualified(&current, &param), Var { addr, width });
                    function.params.push((param, ty));
                }
                funcs.insert(current.clone(), function);
                continue;
            }

            if *line == "end" {
                current.clear();
                continue;
            }

            if let Some(rest) = line.strip_prefix("local ") {
                let (ty, name, count) = parse_decl(rest, "local")?;
                let width = width_of(&ty);
                let addr = allocate(&mut next, width.saturating_mul(count))?;
                vars.insert(qualified(&current, &name), Var { addr, width });
            }

            if allocate_temps {
                for temp in line
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '%' || c == '_'))
                    .filter(|part| part.starts_with('%'))
                {
                    let key = qualified(&current, temp);
                    if !vars.contains_key(&key) {
                        let addr = allocate(&mut next, 2)?;
                        vars.insert(key, Var { addr, width: 2 });
                    }
                }
            }
        }

        Ok(Self {
            vars,
            funcs,
            globals,
            next,
        })
    }

    pub fn resolve(&self, function: &str, name: &str) -> Result<Var, String> {
        self.vars
            .get(&qualified(function, name))
            .or_else(|| self.vars.get(name))
            .cloned()
            .ok_or_else(|| format!("unknown variable {name} in {function}"))
    }

    pub fn scratch(&mut self, width: u16) -> Result<u16, String> {
        allocate(&mut self.next, width)
    }
}

pub fn lines(clir: &str) -> Vec<&str> {
    clir.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(';'))
        .collect()
}

pub fn parse_call(line: &str) -> Result<(Option<&str>, &str, Vec<&str>), String> {
    let (dst, call) = if let Some(eq) = line.find(" = ") {
        let left = &line[5..eq];
        let dst = left
            .split_whitespace()
            .last()
            .ok_or("bad call destination")?;
        (Some(dst), line[eq + 3..].trim())
    } else {
        (None, line.strip_prefix("call ").unwrap_or(line).trim())
    };

    let left = call.find('(').ok_or("bad call")?;
    let right = call.rfind(')').ok_or("bad call")?;
    let name = call[..left].trim();
    let inside = call[left + 1..right].trim();
    let args = if inside.is_empty() {
        Vec::new()
    } else {
        inside.split(',').map(str::trim).collect()
    };
    Ok((dst, name, args))
}
