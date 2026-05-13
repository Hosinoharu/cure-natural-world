#![allow(dead_code)]
//! 定义 JSON 的数据结构（是一棵树）、节点等

use std::{collections::HashMap, fmt::Display};

/// 表示 JSON 树中的节点！
///
/// JSON 中有 6 种数据类型，如果把 true 和 false 当作两个类型就是 7 种。
/// 因为 Rust 中的 enum 的字段可以有复杂类型，所以非常适合表示节点。
#[derive(Debug, Clone)]
pub enum LeptValue {
    Null,
    False,
    True,
    Number(f64),
    String(String),
    Array(Vec<LeptValue>),
    Object(HashMap<String, LeptValue>),
}

// #region 实现标准 trait

impl Default for LeptValue {
    fn default() -> Self {
        LeptValue::Null
    }
}

// 实现 JSON 节点的相等性判断，即重写 == 运算符咯
impl PartialEq for LeptValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LeptValue::Null, LeptValue::Null) => true,
            (LeptValue::False, LeptValue::False) => true,
            (LeptValue::True, LeptValue::True) => true,
            (LeptValue::Number(l), LeptValue::Number(r)) => {
                (l - r).abs() < f64::EPSILON
            }
            (LeptValue::String(l), LeptValue::String(r)) => l == r,
            (LeptValue::Array(l), LeptValue::Array(r)) => l == r,
            (LeptValue::Object(l), LeptValue::Object(r)) => l == r,
            _ => false,
        }
    }
}

impl Eq for LeptValue {}

impl Display for LeptValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeptValue::Null => write!(f, "null"),
            LeptValue::False => write!(f, "false"),
            LeptValue::True => write!(f, "true"),
            LeptValue::Number(v) => write!(f, "{}", v),
            LeptValue::String(v) => write!(f, "\"{}\"", v),
            LeptValue::Array(v) => {
                write!(f, "[")?;
                // 输出数组元素，但最后一个元素后面不加逗号
                for (i, v) in v.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            LeptValue::Object(v) => {
                write!(f, "{{")?;
                // 输出键值对，但最后一个元素后面不加逗号
                for (i, (k, v)) in v.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// 转为 String
impl From<LeptValue> for String {
    fn from(v: LeptValue) -> Self {
        format!("{}", v)
    }
}

// #endregion

// 实现解析 JSON 时用到的一些方法
impl LeptValue {
    /// 创建一个 JSON bool 节点
    pub fn new_bool(v: bool) -> Self {
        if v { LeptValue::True } else { LeptValue::False }
    }

    /// 当节点类型是 Bool 时，返回存储的布尔值
    pub fn get_bool(&self) -> bool {
        match self {
            LeptValue::True => true,
            LeptValue::False => false,
            _ => panic!("Not a bool node!"),
        }
    }

    /// 创建一个 JSON 数值节点
    pub fn new_number(v: f64) -> Self {
        LeptValue::Number(v)
    }

    /// 当节点类型是 Number 时，返回存储的数值
    pub fn get_number(&self) -> f64 {
        match self {
            LeptValue::Number(v) => *v,
            _ => panic!("Not a number node!"),
        }
    }

    /// 创建一个 JSON 字符串节点
    pub fn new_string(s: String) -> Self {
        LeptValue::String(s)
    }

    /// 当节点类型是 String 时，返回存储的字符串
    pub fn get_string(&self) -> &str {
        match self {
            LeptValue::String(s) => s,
            _ => panic!("Not a string node!"),
        }
    }

    /// 创建一个 JSON 数组节点
    pub fn new_array(v: Vec<LeptValue>) -> Self {
        LeptValue::Array(v)
    }

    /// 当节点类型是 Array 时，返回存储的数组
    pub fn get_array(&self) -> &Vec<LeptValue> {
        match self {
            LeptValue::Array(v) => v,
            _ => panic!("Not an array node!"),
        }
    }

    /// 创建一个 JSON 对象节点
    pub fn new_object(v: HashMap<String, LeptValue>) -> Self {
        LeptValue::Object(v)
    }

    /// 当节点类型是 Object 时，返回存储的对象
    pub fn get_object(&self) -> &HashMap<String, LeptValue> {
        match self {
            LeptValue::Object(v) => v,
            _ => panic!("Not an object node!"),
        }
    }

    /// 设置新的值
    pub fn set(&mut self, v: LeptValue) {
        *self = v
    }

    /// 设置新的数组
    pub fn set_array(&mut self, v: Vec<LeptValue>) {
        *self = LeptValue::new_array(v)
    }

    /// 设置新的对象
    pub fn set_object(&mut self, v: HashMap<String, LeptValue>) {
        *self = LeptValue::new_object(v)
    }
}
