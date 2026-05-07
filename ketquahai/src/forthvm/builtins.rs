// ============================================================
// builtins.rs -- Hàm built-in (abs, max, min, sqrt, gcd...)
// ============================================================

use alloc::string::String;
use alloc::vec::Vec;
use crate::forthvm::value::{Value, StringPool, ArrayPool};

/// ID của các built-in functions
pub const BI_ABS:        u32 = 0;
pub const BI_MAX:        u32 = 1;
pub const BI_MIN:        u32 = 2;
pub const BI_SQRT:       u32 = 3;
pub const BI_GCD:        u32 = 4;
pub const BI_SIGN:       u32 = 5;
pub const BI_CLAMP:      u32 = 6;
pub const BI_LENGTH:     u32 = 7;
pub const BI_TYPEOF:     u32 = 8;
pub const BI_STRING:     u32 = 9;
pub const BI_PARSE_INT:  u32 = 10;
pub const BI_UPPERCASE:  u32 = 11;
pub const BI_LOWERCASE:  u32 = 12;
pub const BI_STARTSWITH: u32 = 13;
pub const BI_ENDSWITH:   u32 = 14;
pub const BI_CONTAINS:   u32 = 15;
pub const BI_REPEAT:     u32 = 16;
pub const BI_CHAR:       u32 = 17;
pub const BI_ASCII:      u32 = 18;
pub const BI_ARR_SUM:    u32 = 19;
pub const BI_ARR_MAX:    u32 = 20;
pub const BI_ARR_MIN:    u32 = 21;
pub const BI_ARR_SORT:   u32 = 22;
pub const BI_ARR_REVERSE:u32 = 23;
pub const BI_TICKS:      u32 = 24;
pub const BI_RANDOM:     u32 = 25;
pub const BI_UPTIME:     u32 = 26;
pub const BI_SLEEP:      u32 = 27;
pub const BI_HEAP_FREE:  u32 = 28;
pub const BI_READ_FILE:  u32 = 29;
pub const BI_WRITE_FILE: u32 = 30;
pub const BI_APPEND_FILE:u32 = 31;
pub const BI_FILE_EXISTS:u32 = 32;
pub const BI_PUSH_BANG:  u32 = 33;  // push!(arr, val)
pub const BI_POP_BANG:   u32 = 34;  // pop!(arr)

/// Số tham số cho mỗi built-in
pub fn builtin_param_count(id: u32) -> usize {
    match id {
        BI_ABS | BI_SQRT | BI_SIGN | BI_LENGTH | BI_TYPEOF | BI_STRING |
        BI_PARSE_INT | BI_UPPERCASE | BI_LOWERCASE | BI_CHAR | BI_ASCII |
        BI_ARR_SUM | BI_ARR_MAX | BI_ARR_MIN | BI_ARR_SORT | BI_ARR_REVERSE |
        BI_SLEEP | BI_POP_BANG => 1,

        BI_MAX | BI_MIN | BI_GCD | BI_STARTSWITH | BI_ENDSWITH | BI_CONTAINS |
        BI_REPEAT | BI_WRITE_FILE | BI_PUSH_BANG => 2,

        BI_CLAMP | BI_APPEND_FILE => 3,

        BI_TICKS | BI_RANDOM | BI_UPTIME | BI_HEAP_FREE => 0,

        BI_READ_FILE | BI_FILE_EXISTS => 1,

        _ => 0,
    }
}

/// Tên built-in (cho error messages)
pub fn builtin_name(id: u32) -> &'static str {
    match id {
        BI_ABS => "abs", BI_MAX => "max", BI_MIN => "min",
        BI_SQRT => "sqrt", BI_GCD => "gcd", BI_SIGN => "sign",
        BI_CLAMP => "clamp", BI_LENGTH => "length", BI_TYPEOF => "typeof",
        BI_STRING => "string", BI_PARSE_INT => "parse_int",
        BI_UPPERCASE => "uppercase", BI_LOWERCASE => "lowercase",
        BI_STARTSWITH => "startswith", BI_ENDSWITH => "endswith",
        BI_CONTAINS => "contains", BI_REPEAT => "repeat",
        BI_CHAR => "char", BI_ASCII => "ascii",
        BI_ARR_SUM => "sum", BI_ARR_MAX => "maximum", BI_ARR_MIN => "minimum",
        BI_ARR_SORT => "sort!", BI_ARR_REVERSE => "reverse!",
        BI_TICKS => "ticks", BI_RANDOM => "random", BI_UPTIME => "uptime",
        BI_SLEEP => "sleep", BI_HEAP_FREE => "heap_free",
        BI_READ_FILE => "read_file", BI_WRITE_FILE => "write_file",
        BI_APPEND_FILE => "append_file", BI_FILE_EXISTS => "file_exists",
        BI_PUSH_BANG => "push!", BI_POP_BANG => "pop!",
        _ => "unknown",
    }
}

/// Tra built-in ID theo tên (cho compiler)
pub fn lookup_builtin(name: &str) -> Option<u32> {
    match name {
        "abs" => Some(BI_ABS), "max" => Some(BI_MAX), "min" => Some(BI_MIN),
        "sqrt" => Some(BI_SQRT), "gcd" => Some(BI_GCD), "sign" => Some(BI_SIGN),
        "clamp" => Some(BI_CLAMP), "length" => Some(BI_LENGTH), "typeof" => Some(BI_TYPEOF),
        "string" => Some(BI_STRING), "parse_int" => Some(BI_PARSE_INT),
        "uppercase" => Some(BI_UPPERCASE), "lowercase" => Some(BI_LOWERCASE),
        "startswith" => Some(BI_STARTSWITH), "endswith" => Some(BI_ENDSWITH),
        "contains" => Some(BI_CONTAINS), "repeat" => Some(BI_REPEAT),
        "char" => Some(BI_CHAR), "ascii" => Some(BI_ASCII),
        "sum" => Some(BI_ARR_SUM), "maximum" => Some(BI_ARR_MAX),
        "minimum" => Some(BI_ARR_MIN),
        "ticks" => Some(BI_TICKS), "random" => Some(BI_RANDOM),
        "uptime" => Some(BI_UPTIME), "sleep" => Some(BI_SLEEP),
        "heap_free" => Some(BI_HEAP_FREE),
        "read_file" => Some(BI_READ_FILE), "write_file" => Some(BI_WRITE_FILE),
        "append_file" => Some(BI_APPEND_FILE), "file_exists" => Some(BI_FILE_EXISTS),
        _ => None,
    }
}

/// Thực thi built-in function
/// Tham số đã được push lên stack (tham số cuối ở đỉnh)
pub fn exec_builtin(
    id: u32,
    args: &[Value],
    strings: &mut StringPool,
    arrays: &mut ArrayPool,
) -> Result<Value, &'static str> {
    match id {
        BI_ABS => {
            let n = args[0].as_int().ok_or("abs: expected Int")?;
            Ok(Value::Int(if n < 0 { -n } else { n }))
        }
        BI_MAX => {
            let a = args[0].as_int().ok_or("max: expected Int")?;
            let b = args[1].as_int().ok_or("max: expected Int")?;
            Ok(Value::Int(if a > b { a } else { b }))
        }
        BI_MIN => {
            let a = args[0].as_int().ok_or("min: expected Int")?;
            let b = args[1].as_int().ok_or("min: expected Int")?;
            Ok(Value::Int(if a < b { a } else { b }))
        }
        BI_SQRT => {
            let n = args[0].as_int().ok_or("sqrt: expected Int")?;
            if n < 0 { return Err("sqrt: negative number"); }
            let mut x = n;
            let mut y = (x + 1) / 2;
            while y < x { x = y; y = (x + n / x) / 2; }
            Ok(Value::Int(x))
        }
        BI_GCD => {
            let mut a = args[0].as_int().ok_or("gcd: expected Int")?.abs();
            let mut b = args[1].as_int().ok_or("gcd: expected Int")?.abs();
            while b != 0 { let t = b; b = a % b; a = t; }
            Ok(Value::Int(a))
        }
        BI_SIGN => {
            let n = args[0].as_int().ok_or("sign: expected Int")?;
            Ok(Value::Int(if n > 0 { 1 } else if n < 0 { -1 } else { 0 }))
        }
        BI_CLAMP => {
            let x = args[0].as_int().ok_or("clamp: expected Int")?;
            let lo = args[1].as_int().ok_or("clamp: expected Int")?;
            let hi = args[2].as_int().ok_or("clamp: expected Int")?;
            Ok(Value::Int(if x < lo { lo } else if x > hi { hi } else { x }))
        }
        BI_LENGTH => {
            match &args[0] {
                Value::Str(id) => {
                    let s = strings.get(*id).ok_or("length: invalid string")?;
                    Ok(Value::Int(s.len() as i32))
                }
                Value::Array(id) => {
                    let len = arrays.length(*id).ok_or("length: invalid array")?;
                    Ok(Value::Int(len as i32))
                }
                _ => Err("length: expected String or Array"),
            }
        }
        BI_TYPEOF => {
            let name = args[0].type_name();
            let sid = strings.add_str(name);
            Ok(Value::Str(sid))
        }
        BI_STRING => {
            let s = crate::forthvm::value::format_value(&args[0], strings, arrays);
            let sid = strings.add(s);
            Ok(Value::Str(sid))
        }
        BI_PARSE_INT => {
            if let Value::Str(id) = &args[0] {
                let s = strings.get(*id).ok_or("parse_int: invalid string")?;
                let n: i32 = s.trim().parse().map_err(|_| "parse_int: not a valid number")?;
                Ok(Value::Int(n))
            } else {
                Err("parse_int: expected String")
            }
        }
        BI_UPPERCASE => {
            if let Value::Str(id) = &args[0] {
                let s = strings.get(*id).ok_or("uppercase: invalid string")?;
                let upper: String = s.chars().map(|c| {
                    if c >= 'a' && c <= 'z' { (c as u8 - 32) as char } else { c }
                }).collect();
                let sid = strings.add(upper);
                Ok(Value::Str(sid))
            } else {
                Err("uppercase: expected String")
            }
        }
        BI_LOWERCASE => {
            if let Value::Str(id) = &args[0] {
                let s = strings.get(*id).ok_or("lowercase: invalid string")?;
                let lower: String = s.chars().map(|c| {
                    if c >= 'A' && c <= 'Z' { (c as u8 + 32) as char } else { c }
                }).collect();
                let sid = strings.add(lower);
                Ok(Value::Str(sid))
            } else {
                Err("lowercase: expected String")
            }
        }
        BI_STARTSWITH => {
            if let (Value::Str(s_id), Value::Str(p_id)) = (&args[0], &args[1]) {
                let s = strings.get(*s_id).ok_or("startswith: invalid string")?;
                let p = strings.get(*p_id).ok_or("startswith: invalid prefix")?;
                Ok(Value::Bool(s.starts_with(p)))
            } else {
                Err("startswith: expected (String, String)")
            }
        }
        BI_ENDSWITH => {
            if let (Value::Str(s_id), Value::Str(p_id)) = (&args[0], &args[1]) {
                let s = strings.get(*s_id).ok_or("endswith: invalid string")?;
                let p = strings.get(*p_id).ok_or("endswith: invalid suffix")?;
                Ok(Value::Bool(s.ends_with(p)))
            } else {
                Err("endswith: expected (String, String)")
            }
        }
        BI_CONTAINS => {
            if let (Value::Str(s_id), Value::Str(p_id)) = (&args[0], &args[1]) {
                let s = strings.get(*s_id).ok_or("contains: invalid string")?;
                let p = strings.get(*p_id).ok_or("contains: invalid pattern")?;
                Ok(Value::Bool(s.contains(p)))
            } else {
                Err("contains: expected (String, String)")
            }
        }
        BI_REPEAT => {
            if let (Value::Str(s_id), Value::Int(n)) = (&args[0], &args[1]) {
                let s = strings.get(*s_id).ok_or("repeat: invalid string")?;
                let mut result = String::new();
                for _ in 0..*n { result.push_str(s); }
                let sid = strings.add(result);
                Ok(Value::Str(sid))
            } else {
                Err("repeat: expected (String, Int)")
            }
        }
        BI_CHAR => {
            let n = args[0].as_int().ok_or("char: expected Int")?;
            let c = (n as u8) as char;
            let sid = strings.add(String::from(c));
            Ok(Value::Str(sid))
        }
        BI_ASCII => {
            if let Value::Str(id) = &args[0] {
                let s = strings.get(*id).ok_or("ascii: invalid string")?;
                if let Some(c) = s.chars().next() {
                    Ok(Value::Int(c as i32))
                } else {
                    Err("ascii: empty string")
                }
            } else {
                Err("ascii: expected String")
            }
        }
        BI_ARR_SUM => {
            if let Value::Array(id) = &args[0] {
                let arr = arrays.get(*id).ok_or("sum: invalid array")?;
                let mut total: i32 = 0;
                for v in arr { total += v.as_int().unwrap_or(0); }
                Ok(Value::Int(total))
            } else {
                Err("sum: expected Array")
            }
        }
        BI_ARR_MAX => {
            if let Value::Array(id) = &args[0] {
                let arr = arrays.get(*id).ok_or("maximum: invalid array")?;
                if arr.is_empty() { return Err("maximum: empty array"); }
                let mut mx = i32::MIN;
                for v in arr { let n = v.as_int().unwrap_or(0); if n > mx { mx = n; } }
                Ok(Value::Int(mx))
            } else { Err("maximum: expected Array") }
        }
        BI_ARR_MIN => {
            if let Value::Array(id) = &args[0] {
                let arr = arrays.get(*id).ok_or("minimum: invalid array")?;
                if arr.is_empty() { return Err("minimum: empty array"); }
                let mut mn = i32::MAX;
                for v in arr { let n = v.as_int().unwrap_or(0); if n < mn { mn = n; } }
                Ok(Value::Int(mn))
            } else { Err("minimum: expected Array") }
        }
        BI_ARR_SORT => {
            if let Value::Array(id) = &args[0] {
                let arr = arrays.get_mut(*id).ok_or("sort!: invalid array")?;
                // Bubble sort (đơn giản, đủ cho kernel VM)
                let n = arr.len();
                for i in 0..n {
                    for j in 0..n-1-i {
                        let a = arr[j].as_int().unwrap_or(0);
                        let b = arr[j+1].as_int().unwrap_or(0);
                        if a > b { arr.swap(j, j+1); }
                    }
                }
                Ok(args[0].clone())
            } else { Err("sort!: expected Array") }
        }
        BI_ARR_REVERSE => {
            if let Value::Array(id) = &args[0] {
                let arr = arrays.get_mut(*id).ok_or("reverse!: invalid array")?;
                arr.reverse();
                Ok(args[0].clone())
            } else { Err("reverse!: expected Array") }
        }
        BI_PUSH_BANG => {
            if let Value::Array(id) = &args[0] {
                arrays.push(*id, args[1].clone());
                Ok(Value::Nil)
            } else { Err("push!: expected Array as first arg") }
        }
        BI_POP_BANG => {
            if let Value::Array(id) = &args[0] {
                Ok(arrays.pop(*id).unwrap_or(Value::Nil))
            } else { Err("pop!: expected Array") }
        }
        BI_TICKS => {
            let t = crate::sync::get_ticks() as i32;
            Ok(Value::Int(t))
        }
        BI_RANDOM => {
            let mut buf = [0u8; 4];
            crate::security::fill_random(&mut buf);
            let n = i32::from_le_bytes(buf).abs();
            Ok(Value::Int(n))
        }
        BI_UPTIME => {
            let t = crate::sync::get_ticks();
            let secs = (t / 100) as i32;
            Ok(Value::Int(secs))
        }
        BI_SLEEP => {
            let ms = args[0].as_int().ok_or("sleep: expected Int")?;
            let target = crate::sync::get_ticks() + (ms as u64) / 10;
            while crate::sync::get_ticks() < target {
                core::hint::spin_loop();
            }
            Ok(Value::Nil)
        }
        BI_HEAP_FREE => {
            Ok(Value::Int(0)) // Placeholder — can't easily query heap
        }
        BI_READ_FILE => {
            if let Value::Str(id) = &args[0] {
                let path = strings.get(*id).ok_or("read_file: invalid path")?;
                match crate::fs::read_file(path) {
                    Ok(data) => {
                        let content = core::str::from_utf8(&data).unwrap_or("<binary>");
                        let sid = strings.add(String::from(content));
                        Ok(Value::Str(sid))
                    }
                    Err(_) => Err("read_file: file not found"),
                }
            } else { Err("read_file: expected String path") }
        }
        BI_WRITE_FILE => {
            if let (Value::Str(path_id), Value::Str(content_id)) = (&args[0], &args[1]) {
                let path = strings.get(*path_id).ok_or("write_file: invalid path")?;
                let content = strings.get(*content_id).ok_or("write_file: invalid content")?;
                let path_s = String::from(path);
                let content_s = String::from(content);
                match crate::fs::write_file(&path_s, content_s.as_bytes()) {
                    Ok(()) => Ok(Value::Bool(true)),
                    Err(_) => Err("write_file: failed"),
                }
            } else { Err("write_file: expected (String, String)") }
        }
        BI_APPEND_FILE => {
            // Đọc + nối + ghi lại
            if let (Value::Str(path_id), Value::Str(content_id)) = (&args[0], &args[1]) {
                let path = strings.get(*path_id).ok_or("append_file: invalid path")?;
                let new_content = strings.get(*content_id).ok_or("append_file: invalid content")?;
                let path_s = String::from(path);
                let new_s = String::from(new_content);
                let mut existing = match crate::fs::read_file(&path_s) {
                    Ok(data) => String::from(core::str::from_utf8(&data).unwrap_or("")),
                    Err(_) => String::new(),
                };
                existing.push_str(&new_s);
                match crate::fs::write_file(&path_s, existing.as_bytes()) {
                    Ok(()) => Ok(Value::Bool(true)),
                    Err(_) => Err("append_file: failed"),
                }
            } else { Err("append_file: expected (String, String)") }
        }
        BI_FILE_EXISTS => {
            if let Value::Str(id) = &args[0] {
                let path = strings.get(*id).ok_or("file_exists: invalid path")?;
                let exists = crate::fs::read_file(path).is_ok();
                Ok(Value::Bool(exists))
            } else { Err("file_exists: expected String") }
        }
        _ => Err("unknown builtin"),
    }
}
