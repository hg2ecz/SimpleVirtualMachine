//! Direct CLIR -> Belt16 lowering.
//! CLIR temporaries stay in the eight physical belt slots while possible.
//! A temporary is spilled only before it would fall off the belt or at a
//! control-flow/call boundary.

use super::layout::{Layout, Var, lines, parse_call, parse_fn_header, width_of};

#[derive(Clone)]
struct Slot {
    temp: String,
    stored: bool,
}

struct BeltState {
    slots: [Option<Slot>; 8],
}

impl Default for BeltState {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
        }
    }
}

impl BeltState {
    fn find(&self, temp: &str) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| slot.as_ref().is_some_and(|slot| slot.temp == temp))
    }

    fn forget(&mut self, temp: &str) {
        for slot in &mut self.slots {
            if slot.as_ref().is_some_and(|slot| slot.temp == temp) {
                *slot = None;
            }
        }
    }

    fn push(&mut self, temp: Option<&str>, stored: bool) {
        for i in (1..8).rev() {
            self.slots[i] = self.slots[i - 1].take();
        }
        self.slots[0] = temp.map(|temp| Slot {
            temp: temp.to_string(),
            stored,
        });
    }

    fn clear(&mut self) {
        self.slots = std::array::from_fn(|_| None);
    }
}

fn belt(index: usize) -> String {
    format!("b{index}")
}

fn spill_slot(
    out: &mut String,
    layout: &Layout,
    function: &str,
    state: &mut BeltState,
    index: usize,
) -> Result<(), String> {
    let Some(slot) = state.slots[index].as_mut() else {
        return Ok(());
    };
    if slot.stored {
        return Ok(());
    }
    let addr = layout.resolve(function, &slot.temp)?.addr;
    out.push_str(&format!("    ST16A 0x{addr:04X},b{index}\n"));
    slot.stored = true;
    Ok(())
}

fn prepare_push(
    out: &mut String,
    layout: &Layout,
    function: &str,
    state: &mut BeltState,
) -> Result<(), String> {
    spill_slot(out, layout, function, state, 7)
}

fn push_instruction(
    out: &mut String,
    layout: &Layout,
    function: &str,
    state: &mut BeltState,
    instruction: &str,
    temp: Option<&str>,
    stored: bool,
) -> Result<(), String> {
    prepare_push(out, layout, function, state)?;
    if let Some(temp) = temp {
        state.forget(temp);
    }
    out.push_str("    ");
    out.push_str(instruction);
    out.push('\n');
    state.push(temp, stored);
    Ok(())
}

fn temp_ref(
    out: &mut String,
    layout: &Layout,
    function: &str,
    state: &mut BeltState,
    temp: &str,
) -> Result<usize, String> {
    if let Some(index) = state.find(temp) {
        return Ok(index);
    }
    let addr = layout.resolve(function, temp)?.addr;
    push_instruction(
        out,
        layout,
        function,
        state,
        &format!("LD16A 0x{addr:04X}"),
        Some(temp),
        true,
    )?;
    Ok(0)
}

fn flush(
    out: &mut String,
    layout: &Layout,
    function: &str,
    state: &mut BeltState,
) -> Result<(), String> {
    for index in 0..8 {
        spill_slot(out, layout, function, state, index)?;
    }
    state.clear();
    Ok(())
}

fn store_var(out: &mut String, var: &Var, value_belt: usize) {
    let op = if var.width == 1 { "ST8A" } else { "ST16A" };
    out.push_str(&format!("    {op} 0x{:04X},b{value_belt}\n", var.addr));
}

fn normalize_temp(
    out: &mut String,
    layout: &Layout,
    function: &str,
    dst: &str,
    ty: &str,
    label_id: &mut usize,
    state: &mut BeltState,
) -> Result<(), String> {
    match ty {
        "bool" | "u8" => {
            let mask = if ty == "bool" { "1" } else { "0x00FF" };
            push_instruction(
                out,
                layout,
                function,
                state,
                &format!("LDI {mask}"),
                None,
                false,
            )?;
            let value_was_present = state.find(dst).is_some();
            if !value_was_present {
                temp_ref(out, layout, function, state, dst)?;
            }
            let value = state
                .find(dst)
                .ok_or_else(|| format!("lost Belt temp {dst}"))?;
            let mask_belt = if value_was_present { 0 } else { 1 };
            push_instruction(
                out,
                layout,
                function,
                state,
                &format!("AND {},{}", belt(value), belt(mask_belt)),
                Some(dst),
                false,
            )?;
        }
        "i8" => {
            flush(out, layout, function, state)?;
            let addr = layout.resolve(function, dst)?.addr;
            let n = *label_id;
            *label_id += 1;
            push_instruction(
                out,
                layout,
                function,
                state,
                &format!("LD16A 0x{addr:04X}"),
                None,
                true,
            )?;
            push_instruction(out, layout, function, state, "LDI 0x0080", None, false)?;
            push_instruction(out, layout, function, state, "AND b1,b0", None, false)?;
            out.push_str(&format!("    JZ __cl_belt_i8_pos_{n}\n"));
            state.clear();
            push_instruction(
                out,
                layout,
                function,
                state,
                &format!("LD16A 0x{addr:04X}"),
                None,
                true,
            )?;
            push_instruction(out, layout, function, state, "LDI 0x00FF", None, false)?;
            push_instruction(out, layout, function, state, "AND b1,b0", None, false)?;
            push_instruction(out, layout, function, state, "LDI 0xFF00", None, false)?;
            push_instruction(out, layout, function, state, "OR b1,b0", None, false)?;
            out.push_str(&format!(
                "    ST16A 0x{addr:04X},b0\n    JMP __cl_belt_i8_end_{n}\n"
            ));
            out.push_str(&format!("__cl_belt_i8_pos_{n}:\n"));
            state.clear();
            push_instruction(
                out,
                layout,
                function,
                state,
                &format!("LD16A 0x{addr:04X}"),
                None,
                true,
            )?;
            push_instruction(out, layout, function, state, "LDI 0x00FF", None, false)?;
            push_instruction(out, layout, function, state, "AND b1,b0", None, false)?;
            out.push_str(&format!(
                "    ST16A 0x{addr:04X},b0\n__cl_belt_i8_end_{n}:\n"
            ));
            state.clear();
        }
        _ => {}
    }
    Ok(())
}

fn emit_startup(out: &mut String, layout: &Layout) {
    out.push_str("; Generated by SVM C-Lite for the Belt16 target.\n");
    out.push_str(".load 0x0100\n.entry __start\n\n.proc __start\n");
    for (name, (_, init)) in &layout.globals {
        let Some(value) = init else { continue };
        let var = &layout.vars[name];
        out.push_str(&format!(
            "    LDI {value}\n    {} 0x{:04X},b0\n",
            if var.width == 1 { "ST8A" } else { "ST16A" },
            var.addr
        ));
    }
    out.push_str("    CALL main\n    HALT\n.endproc\n\n");
}

fn emit_compare(
    out: &mut String,
    layout: &Layout,
    function: &str,
    dst: &str,
    op: &str,
    ty: &str,
    left: &str,
    right: &str,
    scratch: [u16; 3],
    label_id: &mut usize,
    state: &mut BeltState,
) -> Result<(), String> {
    flush(out, layout, function, state)?;
    let (la, ra) = if matches!(ty, "i8" | "i16") {
        let bias = if ty == "i8" { "0x0080" } else { "0x8000" };
        let left_addr = layout.resolve(function, left)?.addr;
        let right_addr = layout.resolve(function, right)?.addr;
        push_instruction(
            out,
            layout,
            function,
            state,
            &format!("LD16A 0x{left_addr:04X}"),
            None,
            true,
        )?;
        push_instruction(
            out,
            layout,
            function,
            state,
            &format!("LDI {bias}"),
            None,
            false,
        )?;
        push_instruction(out, layout, function, state, "XOR b1,b0", None, false)?;
        out.push_str(&format!("    ST16A 0x{:04X},b0\n", scratch[0]));
        push_instruction(
            out,
            layout,
            function,
            state,
            &format!("LD16A 0x{right_addr:04X}"),
            None,
            true,
        )?;
        push_instruction(
            out,
            layout,
            function,
            state,
            &format!("LDI {bias}"),
            None,
            false,
        )?;
        push_instruction(out, layout, function, state, "XOR b1,b0", None, false)?;
        out.push_str(&format!("    ST16A 0x{:04X},b0\n", scratch[1]));
        (scratch[0], scratch[1])
    } else {
        (
            layout.resolve(function, left)?.addr,
            layout.resolve(function, right)?.addr,
        )
    };

    state.clear();
    push_instruction(
        out,
        layout,
        function,
        state,
        &format!("LD16A 0x{la:04X}"),
        None,
        true,
    )?;
    push_instruction(
        out,
        layout,
        function,
        state,
        &format!("LD16A 0x{ra:04X}"),
        None,
        true,
    )?;
    push_instruction(out, layout, function, state, "CMP b1,b0", None, false)?;
    let n = *label_id;
    *label_id += 1;
    let t = format!("__cl_belt_cmp_t_{n}");
    let f = format!("__cl_belt_cmp_f_{n}");
    let e = format!("__cl_belt_cmp_e_{n}");
    match op {
        "eq" => out.push_str(&format!("    JZ {t}\n")),
        "ne" => out.push_str(&format!("    JNZ {t}\n")),
        "lt" => out.push_str(&format!("    JNC {t}\n")),
        "ge" => out.push_str(&format!("    JC {t}\n")),
        "le" => out.push_str(&format!("    JNC {t}\n    JZ {t}\n")),
        "gt" => out.push_str(&format!("    JNC {f}\n    JZ {f}\n    JMP {t}\n")),
        _ => return Err(format!("bad compare {op}")),
    }
    let dst_addr = layout.resolve(function, dst)?.addr;
    out.push_str(&format!(
        "{f}:\n    LDI 0\n    ST16A 0x{dst_addr:04X},b0\n    JMP {e}\n{t}:\n    LDI 1\n    ST16A 0x{dst_addr:04X},b0\n{e}:\n"
    ));
    state.clear();
    Ok(())
}

fn emit_signed_divmod(
    out: &mut String,
    layout: &Layout,
    function: &str,
    dst: &str,
    op: &str,
    ty: &str,
    left: &str,
    right: &str,
    scratch: [u16; 3],
    label_id: &mut usize,
    state: &mut BeltState,
) -> Result<(), String> {
    flush(out, layout, function, state)?;
    let [a, b, sign] = scratch;
    let left_addr = layout.resolve(function, left)?.addr;
    let right_addr = layout.resolve(function, right)?.addr;
    out.push_str(&format!(
        "    LD16A 0x{left_addr:04X}\n    ST16A 0x{a:04X},b0\n"
    ));
    out.push_str(&format!(
        "    LD16A 0x{right_addr:04X}\n    ST16A 0x{b:04X},b0\n"
    ));
    let n = *label_id;
    *label_id += 1;
    out.push_str(&format!("    LDI 0\n    ST16A 0x{sign:04X},b0\n"));

    out.push_str(&format!(
        "    LD16A 0x{a:04X}\n    LDI 0\n    CMP b1,b0\n    JNN __cl_belt_sd_a_{n}\n"
    ));
    out.push_str(&format!(
        "    LD16A 0x{a:04X}\n    NEG b0\n    ST16A 0x{a:04X},b0\n"
    ));
    out.push_str(&format!(
        "    LDI 1\n    ST16A 0x{sign:04X},b0\n__cl_belt_sd_a_{n}:\n"
    ));

    out.push_str(&format!(
        "    LD16A 0x{b:04X}\n    LDI 0\n    CMP b1,b0\n    JNN __cl_belt_sd_b_{n}\n"
    ));
    out.push_str(&format!(
        "    LD16A 0x{b:04X}\n    NEG b0\n    ST16A 0x{b:04X},b0\n"
    ));
    if op == "div" {
        out.push_str(&format!(
            "    LD16A 0x{sign:04X}\n    LDI 1\n    XOR b1,b0\n    ST16A 0x{sign:04X},b0\n"
        ));
    }
    out.push_str(&format!("__cl_belt_sd_b_{n}:\n"));
    out.push_str(&format!(
        "    LD16A 0x{a:04X}\n    LD16A 0x{b:04X}\n    {} b1,b0\n",
        if op == "div" { "DIV" } else { "MOD" }
    ));
    let dst_addr = layout.resolve(function, dst)?.addr;
    out.push_str(&format!("    ST16A 0x{dst_addr:04X},b0\n"));
    out.push_str(&format!(
        "    LD16A 0x{sign:04X}\n    LDI 0\n    CMP b1,b0\n    JZ __cl_belt_sd_e_{n}\n"
    ));
    out.push_str(&format!("    LD16A 0x{dst_addr:04X}\n    NEG b0\n    ST16A 0x{dst_addr:04X},b0\n__cl_belt_sd_e_{n}:\n"));
    state.clear();
    normalize_temp(out, layout, function, dst, ty, label_id, state)
}

pub fn lower(clir: &str) -> Result<String, String> {
    let lines = lines(clir);
    let mut layout = Layout::scan(&lines, true)?;
    let scratch = [layout.scratch(2)?, layout.scratch(2)?, layout.scratch(2)?];
    let mut out = String::new();
    emit_startup(&mut out, &layout);
    let mut current = String::new();
    let mut label_id = 0usize;
    let mut state = BeltState::default();

    for line in lines {
        if line.starts_with("global ") || line.starts_with("local ") {
            continue;
        }
        if line.starts_with("fn ") {
            let (name, _) = parse_fn_header(line)?;
            current = name.clone();
            state.clear();
            out.push_str(&format!(".proc {name}\n"));
            continue;
        }
        if line == "end" {
            flush(&mut out, &layout, &current, &mut state)?;
            out.push_str(".endproc\n\n");
            current.clear();
            continue;
        }
        if line.ends_with(':') {
            flush(&mut out, &layout, &current, &mut state)?;
            out.push_str(&format!("{}__{}\n", current, line));
            continue;
        }

        if let Some(rest) = line.strip_prefix("const.") {
            let (_, args) = rest.split_once(' ').ok_or("bad const")?;
            let (dst, value) = args.split_once(',').ok_or("bad const")?;
            push_instruction(
                &mut out,
                &layout,
                &current,
                &mut state,
                &format!("LDI {}", value.trim()),
                Some(dst.trim()),
                false,
            )?;
            continue;
        }
        if let Some(rest) = line.strip_prefix("load.") {
            let (ty, args) = rest.split_once(' ').ok_or("bad load")?;
            let (dst, name) = args.split_once(',').ok_or("bad load")?;
            let var = layout.resolve(&current, name.trim())?;
            push_instruction(
                &mut out,
                &layout,
                &current,
                &mut state,
                &format!(
                    "{} 0x{:04X}",
                    if var.width == 1 { "LD8A" } else { "LD16A" },
                    var.addr
                ),
                Some(dst.trim()),
                false,
            )?;
            normalize_temp(
                &mut out,
                &layout,
                &current,
                dst.trim(),
                ty,
                &mut label_id,
                &mut state,
            )?;
            continue;
        }
        if let Some(rest) = line.strip_prefix("store.") {
            let (_, args) = rest.split_once(' ').ok_or("bad store")?;
            let (name, src) = args.split_once(',').ok_or("bad store")?;
            let value = temp_ref(&mut out, &layout, &current, &mut state, src.trim())?;
            let var = layout.resolve(&current, name.trim())?;
            store_var(&mut out, &var, value);
            continue;
        }
        if let Some(rest) = line.strip_prefix("addr ") {
            let (dst, name) = rest.split_once(',').ok_or("bad addr")?;
            let var = layout.resolve(&current, name.trim())?;
            push_instruction(
                &mut out,
                &layout,
                &current,
                &mut state,
                &format!("LDI 0x{:04X}", var.addr),
                Some(dst.trim()),
                false,
            )?;
            continue;
        }
        if let Some(rest) = line.strip_prefix("index ") {
            let p = rest.split(',').map(str::trim).collect::<Vec<_>>();
            if p.len() != 4 {
                return Err("bad index".into());
            }
            temp_ref(&mut out, &layout, &current, &mut state, p[2])?;
            if p[3] == "2" {
                let index = state
                    .find(p[2])
                    .ok_or_else(|| format!("lost Belt temp {}", p[2]))?;
                push_instruction(
                    &mut out,
                    &layout,
                    &current,
                    &mut state,
                    &format!("SHL1 {}", belt(index)),
                    None,
                    false,
                )?;
                let base_was_present = state.find(p[1]).is_some();
                temp_ref(&mut out, &layout, &current, &mut state, p[1])?;
                let base = state
                    .find(p[1])
                    .ok_or_else(|| format!("lost Belt temp {}", p[1]))?;
                let scaled = if base_was_present { 0 } else { 1 };
                push_instruction(
                    &mut out,
                    &layout,
                    &current,
                    &mut state,
                    &format!("ADD {},{}", belt(base), belt(scaled)),
                    Some(p[0]),
                    false,
                )?;
            } else {
                temp_ref(&mut out, &layout, &current, &mut state, p[1])?;
                let base = state
                    .find(p[1])
                    .ok_or_else(|| format!("lost Belt temp {}", p[1]))?;
                let index = state
                    .find(p[2])
                    .ok_or_else(|| format!("lost Belt temp {}", p[2]))?;
                push_instruction(
                    &mut out,
                    &layout,
                    &current,
                    &mut state,
                    &format!("ADD {},{}", belt(base), belt(index)),
                    Some(p[0]),
                    false,
                )?;
            }
            continue;
        }
        if line.starts_with("loadmem") {
            let volatile = line.starts_with("loadmemv");
            let dot = line.find('.').ok_or("bad loadmem")?;
            let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad loadmem")?;
            let (dst, address) = args.split_once(',').ok_or("bad loadmem")?;
            let address = temp_ref(&mut out, &layout, &current, &mut state, address.trim())?;
            let op = match (volatile, width_of(ty)) {
                (false, 1) => "LD8",
                (false, _) => "LD16",
                (true, 1) => "VLD8",
                (true, _) => "VLD16",
            };
            push_instruction(
                &mut out,
                &layout,
                &current,
                &mut state,
                &format!("{op} [{}]", belt(address)),
                Some(dst.trim()),
                false,
            )?;
            normalize_temp(
                &mut out,
                &layout,
                &current,
                dst.trim(),
                ty,
                &mut label_id,
                &mut state,
            )?;
            continue;
        }
        if line.starts_with("storemem") {
            let volatile = line.starts_with("storememv");
            let dot = line.find('.').ok_or("bad storemem")?;
            let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad storemem")?;
            let (address, value) = args.split_once(',').ok_or("bad storemem")?;
            temp_ref(&mut out, &layout, &current, &mut state, address.trim())?;
            temp_ref(&mut out, &layout, &current, &mut state, value.trim())?;
            let address = state
                .find(address.trim())
                .ok_or_else(|| format!("lost Belt temp {}", address.trim()))?;
            let value = state
                .find(value.trim())
                .ok_or_else(|| format!("lost Belt temp {}", value.trim()))?;
            let op = match (volatile, width_of(ty)) {
                (false, 1) => "ST8",
                (false, _) => "ST16",
                (true, 1) => "VST8",
                (true, _) => "VST16",
            };
            out.push_str(&format!("    {op} [{}],{}\n", belt(address), belt(value)));
            continue;
        }
        if let Some(temp) = line.strip_prefix("drop ") {
            state.forget(temp.trim());
            continue;
        }
        if let Some(label) = line.strip_prefix("jmp ") {
            flush(&mut out, &layout, &current, &mut state)?;
            out.push_str(&format!("    JMP {current}__{}\n", label.trim()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("jz ") {
            let (value, label) = rest.split_once(',').ok_or("bad jz")?;
            flush(&mut out, &layout, &current, &mut state)?;
            let value = temp_ref(&mut out, &layout, &current, &mut state, value.trim())?;
            push_instruction(
                &mut out,
                &layout,
                &current,
                &mut state,
                &format!("PASS {}", belt(value)),
                None,
                false,
            )?;
            out.push_str(&format!("    JZ {current}__{}\n", label.trim()));
            state.clear();
            continue;
        }
        if line == "ret" {
            out.push_str("    RET\n");
            state.clear();
            continue;
        }
        if let Some(value) = line.strip_prefix("ret ") {
            let value = temp_ref(&mut out, &layout, &current, &mut state, value.trim())?;
            out.push_str(&format!("    ZST16 0x00,{}\n    RET\n", belt(value)));
            state.clear();
            continue;
        }
        if line.starts_with("call") {
            let (dst, name, args) = parse_call(line)?;
            let function = layout
                .funcs
                .get(name)
                .ok_or_else(|| format!("unknown call target {name}"))?;
            if args.len() != function.params.len() {
                return Err(format!("call arity mismatch for {name}"));
            }
            flush(&mut out, &layout, &current, &mut state)?;
            for (arg, (param, _)) in args.iter().zip(&function.params) {
                let value = temp_ref(&mut out, &layout, &current, &mut state, arg)?;
                let slot = layout.resolve(name, param)?;
                store_var(&mut out, &slot, value);
            }
            out.push_str(&format!("    CALL {name}\n"));
            state.clear();
            if let Some(dst) = dst {
                push_instruction(
                    &mut out,
                    &layout,
                    &current,
                    &mut state,
                    "ZLD16 0x00",
                    Some(dst),
                    false,
                )?;
            }
            continue;
        }

        if let Some((op_ty, args)) = line.split_once(' ') {
            if let Some((op, ty)) = op_ty.split_once('.') {
                if matches!(op, "neg" | "not") {
                    let (dst, src) = args.split_once(',').ok_or("bad unary")?;
                    let src = temp_ref(&mut out, &layout, &current, &mut state, src.trim())?;
                    push_instruction(
                        &mut out,
                        &layout,
                        &current,
                        &mut state,
                        &format!("{} {}", if op == "neg" { "NEG" } else { "NOT" }, belt(src)),
                        Some(dst.trim()),
                        false,
                    )?;
                    normalize_temp(
                        &mut out,
                        &layout,
                        &current,
                        dst.trim(),
                        ty,
                        &mut label_id,
                        &mut state,
                    )?;
                    continue;
                }
                let p = args.split(',').map(str::trim).collect::<Vec<_>>();
                if p.len() != 3 {
                    return Err(format!("bad binary CLIR: {line}"));
                }
                if matches!(op, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
                    emit_compare(
                        &mut out,
                        &layout,
                        &current,
                        p[0],
                        op,
                        ty,
                        p[1],
                        p[2],
                        scratch,
                        &mut label_id,
                        &mut state,
                    )?;
                } else if matches!(op, "div" | "mod") && matches!(ty, "i8" | "i16") {
                    emit_signed_divmod(
                        &mut out,
                        &layout,
                        &current,
                        p[0],
                        op,
                        ty,
                        p[1],
                        p[2],
                        scratch,
                        &mut label_id,
                        &mut state,
                    )?;
                } else {
                    temp_ref(&mut out, &layout, &current, &mut state, p[1])?;
                    temp_ref(&mut out, &layout, &current, &mut state, p[2])?;
                    let left = state
                        .find(p[1])
                        .ok_or_else(|| format!("lost Belt temp {}", p[1]))?;
                    let right = state
                        .find(p[2])
                        .ok_or_else(|| format!("lost Belt temp {}", p[2]))?;
                    let asm = match op {
                        "add" => "ADD",
                        "sub" => "SUB",
                        "mul" => "MUL",
                        "div" => "DIV",
                        "mod" => "MOD",
                        "and" => "AND",
                        "or" => "OR",
                        "xor" => "XOR",
                        "shl" => "SHL",
                        "shr" => "SHR",
                        _ => return Err(format!("unsupported Belt CLIR op {op}")),
                    };
                    push_instruction(
                        &mut out,
                        &layout,
                        &current,
                        &mut state,
                        &format!("{asm} {},{}", belt(left), belt(right)),
                        Some(p[0]),
                        false,
                    )?;
                    normalize_temp(
                        &mut out,
                        &layout,
                        &current,
                        p[0],
                        ty,
                        &mut label_id,
                        &mut state,
                    )?;
                }
                continue;
            }
        }
        return Err(format!("unsupported Belt CLIR line: {line}"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_recent_temporaries_on_physical_belt() {
        let clir = "fn main() -> u16\n  const.u16 %0, 7\n  const.u16 %1, 5\n  mul.u16 %2, %0, %1\n  ret %2\nend\n";
        let asm = lower(clir).unwrap();
        assert!(asm.contains("MUL b1,b0"));
        assert!(
            !asm.lines()
                .any(|line| line.trim_start().starts_with("ST16A 0x") && line.contains(",b7"))
        );
    }

    #[test]
    fn spills_only_when_a_live_temp_would_fall_off_b7() {
        let mut clir = String::from("fn main() -> u16\n");
        for n in 0..9 {
            clir.push_str(&format!("  const.u16 %{n}, {n}\n"));
        }
        clir.push_str("  add.u16 %9, %0, %8\n  ret %9\nend\n");
        let asm = lower(&clir).unwrap();
        assert!(asm.contains("ST16A"));
    }
}
