// ============================================================
// value.rs -- Kiểu giá trị động (Dynamic Value)
// Hỗ trợ: Int, Bool, String, Array, Nil
// ============================================================

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// ID tham chiếu chuỗi trong pool
pub type StrId = u32;
/// ID tham chiếu mảng trong pool
pub type ArrId = u32;

/// Giá trị động — có thể là số, bool, chuỗi, mảng, hoặc nil
#[derive(Clone, Debug)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Str(StrId),
    Array(ArrId),
    Nil,
}

impl Value {
    /// Chuyển thành i32 (cho các phép tính số học)
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Kiểm tra "truthy" — false/0/nil = false, còn lại = true
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(n) => *n != 0,
            Value::Bool(b) => *b,
            Value::Str(_) => true,
            Value::Array(_) => true,
            Value::Nil => false,
        }
    }

    /// Tên kiểu dữ liệu
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Bool(_) => "Bool",
            Value::Str(_) => "String",
            Value::Array(_) => "Array",
            Value::Nil => "Nil",
        }
    }

    /// Giá trị zero/default
    pub fn zero() -> Self { Value::Int(0) }
}

impl Default for Value {
    fn default() -> Self { Value::Nil }
}

/// Pool chuỗi — lưu trữ tất cả chuỗi, tham chiếu bằng StrId
pub struct StringPool {
    strings: Vec<String>,
}

impl StringPool {
    pub fn new() -> Self {
        StringPool { strings: Vec::new() }
    }

    /// Thêm chuỗi mới vào pool, trả về StrId
    pub fn add(&mut self, s: String) -> StrId {
        let id = self.strings.len() as StrId;
        self.strings.push(s);
        id
    }

    /// Thêm &str vào pool
    pub fn add_str(&mut self, s: &str) -> StrId {
        self.add(String::from(s))
    }

    /// Lấy chuỗi theo ID
    pub fn get(&self, id: StrId) -> Option<&str> {
        self.strings.get(id as usize).map(|s| s.as_str())
    }

    /// Số chuỗi trong pool
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Reset pool
    pub fn reset(&mut self) {
        self.strings.clear();
    }
}

/// Pool mảng — lưu trữ tất cả mảng, tham chiếu bằng ArrId
pub struct ArrayPool {
    arrays: Vec<Vec<Value>>,
}

impl ArrayPool {
    pub fn new() -> Self {
        ArrayPool { arrays: Vec::new() }
    }

    /// Tạo mảng mới rỗng, trả về ArrId
    pub fn create(&mut self) -> ArrId {
        let id = self.arrays.len() as ArrId;
        self.arrays.push(Vec::new());
        id
    }

    /// Tạo mảng với nội dung cho trước
    pub fn create_with(&mut self, items: Vec<Value>) -> ArrId {
        let id = self.arrays.len() as ArrId;
        self.arrays.push(items);
        id
    }

    /// Lấy mảng theo ID (immutable)
    pub fn get(&self, id: ArrId) -> Option<&Vec<Value>> {
        self.arrays.get(id as usize)
    }

    /// Lấy mảng theo ID (mutable)
    pub fn get_mut(&mut self, id: ArrId) -> Option<&mut Vec<Value>> {
        self.arrays.get_mut(id as usize)
    }

    /// Push phần tử vào mảng
    pub fn push(&mut self, id: ArrId, val: Value) -> bool {
        if let Some(arr) = self.arrays.get_mut(id as usize) {
            arr.push(val);
            true
        } else {
            false
        }
    }

    /// Pop phần tử từ mảng
    pub fn pop(&mut self, id: ArrId) -> Option<Value> {
        self.arrays.get_mut(id as usize)?.pop()
    }

    /// Đọc phần tử (1-indexed giống Julia)
    pub fn index_get(&self, id: ArrId, index: i32) -> Option<Value> {
        let arr = self.arrays.get(id as usize)?;
        if index < 1 || index as usize > arr.len() {
            None
        } else {
            Some(arr[(index - 1) as usize].clone())
        }
    }

    /// Ghi phần tử (1-indexed giống Julia)
    pub fn index_set(&mut self, id: ArrId, index: i32, val: Value) -> bool {
        if let Some(arr) = self.arrays.get_mut(id as usize) {
            if index >= 1 && (index as usize) <= arr.len() {
                arr[(index - 1) as usize] = val;
                return true;
            }
        }
        false
    }

    /// Độ dài mảng
    pub fn length(&self, id: ArrId) -> Option<usize> {
        self.arrays.get(id as usize).map(|a| a.len())
    }

    /// Reset pool
    pub fn reset(&mut self) {
        self.arrays.clear();
    }
}

/// Format Value thành chuỗi hiển thị
pub fn format_value(val: &Value, strings: &StringPool, arrays: &ArrayPool) -> String {
    match val {
        Value::Int(n) => format!("{}", n),
        Value::Bool(b) => if *b { String::from("true") } else { String::from("false") },
        Value::Str(id) => {
            match strings.get(*id) {
                Some(s) => String::from(s),
                None => String::from("<invalid string>"),
            }
        }
        Value::Array(id) => {
            match arrays.get(*id) {
                Some(arr) => {
                    let mut s = String::from("[");
                    for (i, v) in arr.iter().enumerate() {
                        if i > 0 { s.push_str(", "); }
                        s.push_str(&format_value(v, strings, arrays));
                    }
                    s.push(']');
                    s
                }
                None => String::from("<invalid array>"),
            }
        }
        Value::Nil => String::from("nil"),
    }
}
