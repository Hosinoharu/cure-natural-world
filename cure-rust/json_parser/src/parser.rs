//! 解析 JSON 文本

use crate::node::*;
use std::collections::HashMap;

/// 表示解析 JSON 字符串后最终的状态
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParseStatus {
    Ok,
    ExpectValue,
    InvalidValue,
    RootNotSingular,
    /// 解析字符串时，缺少了引号
    MissingQuotationMark,
    /// 解析字符串时，遇到了非法的转义字符
    InvalidStringEscape,
    /// 解析字符串时，遇到了非法的字符
    InvalidStringChar,
    /// 解析 unicode 字符串时，遇到了非法的 unicode 转义字符
    InvalidUnicodeHex,
    /// 解析 unicode 字符串时，遇到了非法的 unicode surrogate 字符
    InvalidUnicodeSurrogate,
    /// 解析数组时，缺少了逗号或右方括号
    MissCommaOrSquareBracket,
    /// 解析对象时，缺少了 key
    MissKey,
    /// 解析对象时，缺少了冒号
    MissColon,
    /// 解析对象时，缺少了逗号或右花括号
    MissCommaOrCurlyBracket,
}

/// 一个 JSON 解析器。
/// 由于 JSON 语法特别简单，我们不需要写分词器（tokenizer）,
/// 直接逐个字符解析即可
pub struct LeptParser {
    /// 当前解析的 JSON 字符
    chars: String,
    /// 当前解析的字符位置
    pos: usize,
}

// 实现内部的字符串读写
impl LeptParser {
    /// 获取当前的字符，不增加索引
    fn next(&mut self) -> Option<char> {
        self.chars[self.pos..].chars().next()
    }

    /// 已经解析完成！
    fn eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    /// 解析字符串时，判断当前位置后方是否为 s 字符串
    fn starts_with(&self, s: &str) -> bool {
        self.chars[self.pos..].starts_with(s)
    }

    /// 确保解析开始时，当前位置后方是否为 s 字符串，同时增加索引跳过 s 字符串。
    /// 如果当前位置后方不是 s 字符串，则 panic！
    fn expect(&mut self, s: &str) {
        assert!(self.starts_with(s), "expect {}", s);
        self.pos += s.len();
    }
}

// 实现解析 JSON 值
impl LeptParser {
    /// 传入一个空的 JSON 节点，解析一个 JSON 字符串！返回解析后的节点树、解析后状态
    pub fn parse(node: &mut LeptValue, json: &str) -> ParseStatus {
        let mut parser = LeptParser {
            chars: json.to_string(),
            pos: 0,
        };
        // 无论如何，节点都要初始化为 null
        node.set(LeptValue::default());
        parser.parse_whitespace();
        let status = parser.parse_value(node);
        // 成功解析完成当前一个节点之后，还需要继续哟
        if status == ParseStatus::Ok {
            parser.parse_whitespace();
            if !parser.eof() {
                node.set(LeptValue::default());
                return ParseStatus::RootNotSingular;
            }
        }
        status
    }

    /** 解析各种 JSON 值

       n ➔ null：碰到 n，看后续是否为 null

       t ➔ true：碰到 t，看后续是否为 true

       f ➔ false：碰到 f，看后续是否为 false

       " ➔ string：碰到 "，则说明是字符串

       0-9/- ➔ number：碰到数字，则说明是数字

       [ ➔ array：碰到 [，则说明是数组

       { ➔ object：碰到 {，则说明是对象
    */
    fn parse_value(&mut self, node: &mut LeptValue) -> ParseStatus {
        // 说明解析器已经到达字符串末尾了
        if self.eof() {
            return ParseStatus::ExpectValue;
        }
        match self.next().unwrap() {
            'n' => self.parse_literal(node, "null", LeptValue::Null),
            't' => self.parse_literal(node, "true", LeptValue::new_bool(true)),
            'f' => {
                self.parse_literal(node, "false", LeptValue::new_bool(false))
            }
            '"' => self.parse_string(node),
            '[' => self.parse_array(node),
            '{' => self.parse_object(node),
            _ => self.parse_number(node),
        }
    }

    /// 解析 whitespace，跳过它们
    fn parse_whitespace(&mut self) {
        while !self.eof() && self.next().unwrap().is_whitespace() {
            self.pos += 1;
        }
    }

    /// 解析 null、true、false 等字面量值
    fn parse_literal(
        &mut self,
        node: &mut LeptValue,
        literal: &str,
        literal_value: LeptValue,
    ) -> ParseStatus {
        if self.starts_with(literal) {
            self.pos += literal.len();
            node.set(literal_value);
            return ParseStatus::Ok;
        } else {
            return ParseStatus::InvalidValue;
        }
    }

    /// 解析 JSON 数值
    fn parse_number(&mut self, node: &mut LeptValue) -> ParseStatus {
        // 存储解析出来的、为数值字符串
        let mut num = String::new();
        let mut current = self.next();

        // 如 -123
        if current == Some('-') {
            num.push('-');
            self.pos += 1;
            current = self.next();
        }

        // 数字以 0 开头，可能是 0123 或者小数 0.123
        if current == Some('0') {
            num.push('0');
            self.pos += 1;
            current = self.next();
        }
        // 解析正常的数字，如 123
        else {
            // 第一个数字必须是 1-9 的数字
            if current.is_none() || !is_digit_19(current.unwrap()) {
                return ParseStatus::InvalidValue;
            }
            // 解析数字，直到碰到非数字字符
            while !self.eof() {
                if current.is_none() || !current.unwrap().is_ascii_digit() {
                    break;
                }
                num.push(current.unwrap());
                self.pos += 1;
                current = self.next();
            }
        }

        // 解析小数点
        if current == Some('.') {
            num.push('.');
            self.pos += 1;
            current = self.next();
            // 小数点后必须跟数字
            if current.is_none() || !current.unwrap().is_ascii_digit() {
                return ParseStatus::InvalidValue;
            }
            // 解析小数点后的数字
            while !self.eof() {
                if current.is_none() || !current.unwrap().is_ascii_digit() {
                    break;
                }
                num.push(current.unwrap());
                self.pos += 1;
                current = self.next();
            }
        }

        // 解析指数部分
        if current == Some('e') || current == Some('E') {
            num.push(current.unwrap());
            self.pos += 1;
            current = self.next();
            // 指数部分必须跟一个符号
            if current == Some('+') || current == Some('-') {
                num.push(current.unwrap());
                self.pos += 1;
                current = self.next();
            }

            // 指数部分必须跟数字
            if current.is_none() || !current.unwrap().is_ascii_digit() {
                return ParseStatus::InvalidValue;
            }
            while !self.eof() {
                if current.is_none() || !current.unwrap().is_ascii_digit() {
                    break;
                }
                num.push(current.unwrap());
                self.pos += 1;
                current = self.next();
            }
        }

        // 将解析出来的数值字符串转换为浮点数
        match num.parse::<f64>() {
            Ok(num) => {
                node.set(LeptValue::new_number(num));
                ParseStatus::Ok
            }
            Err(_) => ParseStatus::InvalidValue,
        }
    }

    /// 解析字符串
    fn parse_string(&mut self, node: &mut LeptValue) -> ParseStatus {
        let mut result = String::new();
        let status = self.parse_raw_string(&mut result);
        if status == ParseStatus::Ok {
            node.set(LeptValue::new_string(result));
        }
        status
    }

    /// 解析一个原始字符串，为了方便后续解析出对象的 key
    fn parse_raw_string(&mut self, result: &mut String) -> ParseStatus {
        // 跳过开头的引号
        self.expect("\"");
        loop {
            let current = self.next();
            self.pos += 1;
            match current {
                // 碰到结束的引号，返回解析成功
                Some('"') => return ParseStatus::Ok,
                // 处理转义字符
                Some('\\') => {
                    // 跳过反斜杠，并向后看一位
                    let current = self.next();
                    self.pos += 1;
                    match current {
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some('/') => result.push('/'),
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('t') => result.push('\t'),
                        Some('b') => result.push('\x08'), // \b
                        Some('f') => result.push('\x0c'), // \f
                        // 处理 unicode，如 \u4e2d
                        Some('u') => {
                            // 存储解析出来的 unicode 字符串
                            let mut unicode = String::new();
                            // 往后面看 4 位，必须是 16 进制的数字
                            for _ in 0..4 {
                                let current = self.next();
                                self.pos += 1;
                                match current {
                                    Some(c) if c.is_ascii_hexdigit() => {
                                        unicode.push(c)
                                    }
                                    _ => {
                                        return ParseStatus::InvalidUnicodeHex;
                                    }
                                }
                            }
                            // 将 unicode 字符串转换为字符
                            match u32::from_str_radix(&unicode, 16) {
                                Ok(code) => {
                                    result.push(char::from_u32(code).unwrap())
                                }
                                Err(_) => {
                                    return ParseStatus::InvalidUnicodeSurrogate;
                                }
                            }
                        }
                        _ => return ParseStatus::InvalidStringEscape,
                    }
                }
                Some(c) => {
                    if c < '\x20' {
                        return ParseStatus::InvalidStringChar;
                    }
                    result.push(c)
                }
                None => return ParseStatus::MissingQuotationMark,
            }
        }
    }

    /// 解析数组
    fn parse_array(&mut self, node: &mut LeptValue) -> ParseStatus {
        let mut result: Vec<LeptValue> = Vec::new();
        // 跳过开头的 [ 符号
        self.expect("[");

        // 判断空数组啦
        self.parse_whitespace();
        match self.next() {
            Some(']') => {
                self.pos += 1;
                node.set_array(result);
                return ParseStatus::Ok;
            }
            // 说明后面是其它有效字符，不处理
            Some(_) => (),
            None => return ParseStatus::MissCommaOrSquareBracket,
        }

        // 非空数组，解析数组中的元素
        loop {
            // 创建空的节点，保存后续的解析结果
            let mut value = LeptValue::default();
            let status = self.parse_value(&mut value);
            if status != ParseStatus::Ok {
                return status;
            }
            result.push(value);

            // 解析一个元素之后，后续还得判断！
            self.parse_whitespace();
            let current = self.next();
            self.pos += 1;
            match current {
                // 数组元素分隔符
                Some(',') => {
                    self.parse_whitespace();
                }
                // 结束符
                Some(']') => {
                    node.set_array(result);
                    return ParseStatus::Ok;
                }
                _ => return ParseStatus::MissCommaOrSquareBracket,
            }
        }
    }

    /// 解析对象
    fn parse_object(&mut self, node: &mut LeptValue) -> ParseStatus {
        let mut result: HashMap<String, LeptValue> = HashMap::new();
        // 跳过开头的 { 符号
        self.expect("{");

        // 判断空对象啦
        self.parse_whitespace();
        match self.next() {
            Some('}') => {
                self.pos += 1;
                node.set_object(result);
                return ParseStatus::Ok;
            }
            // 说明后面是其它有效字符，不处理
            Some(_) => (),
            None => return ParseStatus::MissCommaOrCurlyBracket,
        }

        // 开始解析键值对
        loop {
            // 记录解析出来的 key
            let mut key = String::new();
            self.parse_whitespace();
            match self.next() {
                // 第一个字符应该是字符串开头的 " 哟
                Some('"') => {
                    let status = self.parse_raw_string(&mut key);
                    if status != ParseStatus::Ok {
                        return status;
                    }
                }
                _ => return ParseStatus::MissKey,
            }

            // 解析冒号
            self.parse_whitespace();
            match self.next() {
                Some(':') => {
                    self.pos += 1;
                }
                _ => return ParseStatus::MissColon,
            }

            // 解析值
            let mut value = LeptValue::default();
            self.parse_whitespace();
            let status = self.parse_value(&mut value);
            if status != ParseStatus::Ok {
                return status;
            }
            result.insert(key, value);

            // 解析一个键值对之后，后续还得判断！
            self.parse_whitespace();
            let current = self.next();
            self.pos += 1;
            match current {
                // 对象元素分隔符
                Some(',') => {
                    self.parse_whitespace();
                }
                // 结束符
                Some('}') => {
                    node.set_object(result);
                    return ParseStatus::Ok;
                }
                _ => return ParseStatus::MissCommaOrCurlyBracket,
            }
        }
    }
}

/// 判断一个字符串是否为 1-9 的数字
fn is_digit_19(c: char) -> bool {
    c >= '1' && c <= '9'
}

#[cfg(test)]
mod tests {
    use super::*;

    // #region 一些辅助测试的宏

    /// 创建解析 JSON 错误的示例。`test_error!(ParseStatus::InvalidValue, "null x")`
    ///
    /// 第一个参数是期望的解析状态，第二个参数是 JSON 字符串
    macro_rules! test_error {
        ($error:expr, $json:expr) => {{
            let mut node = LeptValue::new_bool(true);
            assert_eq!($error, LeptParser::parse(&mut node, $json));
            assert_eq!(LeptValue::Null, node);
        }};
    }

    /// 创建解析 JSON 数值的示例。`test_number!(0.1, "0.2")`
    ///
    /// 第一个参数是期望的数值，第二个参数是 JSON 字符串
    macro_rules! test_number {
        ($expected:expr, $json:expr) => {{
            let mut node = LeptValue::default();
            LeptParser::parse(&mut node, $json);
            assert_eq!(ParseStatus::Ok, LeptParser::parse(&mut node, $json));
            assert_eq!($expected, node.get_number());
        }};
    }

    /// 创建解析 JSON 字符串的示例。`test_string!(expect, json)`
    macro_rules! test_string {
        ($expected:expr, $json:expr) => {{
            let mut node = LeptValue::default();
            LeptParser::parse(&mut node, $json);
            assert_eq!(ParseStatus::Ok, LeptParser::parse(&mut node, $json));
            assert_eq!($expected, node.get_string());
        }};
    }

    // #endregion

    /// 测试解析 json 中的 null 值
    #[test]
    fn test_parse_null() {
        // 创建一个 True 类型的节点咯
        let mut node = LeptValue::new_bool(true);
        // 解析 null 值
        assert_eq!(ParseStatus::Ok, LeptParser::parse(&mut node, "null"));
        assert_eq!(LeptValue::Null, node);
    }

    /// 测试解析 json 中的 true 值
    #[test]
    fn test_parse_true() {
        let mut node = LeptValue::default();
        assert_eq!(ParseStatus::Ok, LeptParser::parse(&mut node, "true"));
        assert_eq!(LeptValue::True, node);
    }

    /// 测试解析 json 中的 false 值
    #[test]
    fn test_parse_false() {
        let mut node = LeptValue::default();
        assert_eq!(ParseStatus::Ok, LeptParser::parse(&mut node, "false"));
        assert_eq!(LeptValue::False, node);
    }

    /// 测试解析 json 中的空字符串
    #[test]
    fn test_parse_expect_value() {
        test_error!(ParseStatus::ExpectValue, "");
        test_error!(ParseStatus::ExpectValue, " ");
    }

    /// 测试解析 json 中的非法值
    #[test]
    fn test_parse_invalid_value() {
        test_error!(ParseStatus::InvalidValue, "nul");
        test_error!(ParseStatus::InvalidValue, "?");

        /* invalid number */
        test_error!(ParseStatus::InvalidValue, "+0");
        test_error!(ParseStatus::InvalidValue, "+1");
        test_error!(ParseStatus::InvalidValue, ".123"); /* at least one digit before '.' */
        test_error!(ParseStatus::InvalidValue, "1."); /* at least one digit after '.' */
        test_error!(ParseStatus::InvalidValue, "INF");
        test_error!(ParseStatus::InvalidValue, "inf");
        test_error!(ParseStatus::InvalidValue, "NAN");
        test_error!(ParseStatus::InvalidValue, "nan");
    }

    /// 测试解析 json 中的多余字符
    #[test]
    fn test_parse_root_not_singular() {
        test_error!(ParseStatus::RootNotSingular, "null x");

        /* invalid number */
        test_error!(ParseStatus::RootNotSingular, "0123"); /* after zero should be '.' or nothing */
        test_error!(ParseStatus::RootNotSingular, "0x0");
        test_error!(ParseStatus::RootNotSingular, "0x123");
    }

    /// 测试解析 json 中的数值
    #[test]
    fn test_parse_number() {
        test_number!(0.0, "0");
        test_number!(0.0, "-0");
        test_number!(0.0, "-0.0");
        test_number!(1.0, "1");
        test_number!(-1.0, "-1");
        test_number!(1.5, "1.5");
        test_number!(-1.5, "-1.5");
        test_number!(3.1416, "3.1416");
        test_number!(1E10, "1E10");
        test_number!(1e10, "1e10");
        test_number!(1E+10, "1E+10");
        test_number!(1E-10, "1E-10");
        test_number!(-1E10, "-1E10");
        test_number!(-1e10, "-1e10");
        test_number!(-1E+10, "-1E+10");
        test_number!(-1E-10, "-1E-10");
        test_number!(1.234E+10, "1.234E+10");
        test_number!(1.234E-10, "1.234E-10");
        test_number!(0.0, "1e-10000"); /* must underflow */

        test_number!(1.0000000000000002, "1.0000000000000002"); /* the smallest number > 1 */
        test_number!(4.9406564584124654e-324, "4.9406564584124654e-324"); /* minimum denormal */
        test_number!(-4.9406564584124654e-324, "-4.9406564584124654e-324");
        test_number!(2.2250738585072009e-308, "2.2250738585072009e-308"); /* Max subnormal double */
        test_number!(-2.2250738585072009e-308, "-2.2250738585072009e-308");
        test_number!(2.2250738585072014e-308, "2.2250738585072014e-308"); /* Min normal positive double */
        test_number!(-2.2250738585072014e-308, "-2.2250738585072014e-308");
        test_number!(1.7976931348623157e+308, "1.7976931348623157e+308"); /* Max double */
        test_number!(-1.7976931348623157e+308, "-1.7976931348623157e+308");
    }

    /// 测试解析 json 中的字符串
    #[test]
    fn test_parse_string() {
        test_string!("", "\"\"");
        test_string!("Hello", "\"Hello\"");
        test_string!("Hello\nWorld", "\"Hello\\nWorld\"");

        test_error!(ParseStatus::MissingQuotationMark, "\"");
        test_error!(ParseStatus::MissingQuotationMark, "\"abc");

        test_error!(ParseStatus::InvalidStringEscape, "\"\\v\"");
        test_error!(ParseStatus::InvalidStringEscape, "\"\\'\"");
        test_error!(ParseStatus::InvalidStringEscape, "\"\\0\"");
        test_error!(ParseStatus::InvalidStringEscape, "\"\\x12\"");

        test_error!(ParseStatus::InvalidStringChar, "\"\x01\"");
        test_error!(ParseStatus::InvalidStringChar, "\"\x1F\"");

        test_string!("Hello\0World", "\"Hello\\u0000World\"");
        test_string!("\x24", "\"\\u0024\""); /* Dollar sign U+0024 */
        // test_string!(r"\xC2\xA2", "\"\\u00A2\""); /* Cents sign U+00A2 */
        // test_string!(r"\xE2\x82\xAC", "\"\\u20AC\""); /* Euro sign U+20AC */
        // test_string!(r"\xF0\x9D\x84\x9E", "\"\\uD834\\uDD1E\""); /* G clef sign U+1D11E */
        // test_string!(r"\xF0\x9D\x84\x9E", "\"\\ud834\\udd1e\""); /* G clef sign U+1D11E */
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u0\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u01\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u012\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u/000\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\uG000\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u0/00\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u0G00\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u00/0\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u00G0\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u000/\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u000G\"");
        test_error!(ParseStatus::InvalidUnicodeHex, "\"\\u 123\"");
    }

    /// 测试解析 json 中的数组
    #[test]
    fn test_parse_array() {
        let mut node = LeptValue::default();

        // 测试空数组
        LeptParser::parse(&mut node, "[  ]");
        assert_eq!(LeptValue::new_array(vec![]), node);

        // 测试非空数组
        LeptParser::parse(&mut node, "[ null , false , true , 123 , \"abc\" ]");
        assert_eq!(
            LeptValue::new_array(vec![
                LeptValue::default(),
                LeptValue::new_bool(false),
                LeptValue::new_bool(true),
                LeptValue::new_number(123.0),
                LeptValue::new_string("abc".to_string())
            ]),
            node
        );

        // 测试嵌套数组
        LeptParser::parse(
            &mut node,
            "[ [ ] , [ 0 ] , [ 0 , 1 ] , [ 0 , 1 , 2 ] ]",
        );
        assert_eq!(
            LeptValue::new_array(vec![
                LeptValue::new_array(vec![]),
                LeptValue::new_array(vec![LeptValue::new_number(0.0)]),
                LeptValue::new_array(vec![
                    LeptValue::new_number(0.0),
                    LeptValue::new_number(1.0),
                ]),
                LeptValue::new_array(vec![
                    LeptValue::new_number(0.0),
                    LeptValue::new_number(1.0),
                    LeptValue::new_number(2.0),
                ]),
            ]),
            node
        );

        // 测试错误
        test_error!(ParseStatus::MissCommaOrSquareBracket, "[");
        test_error!(ParseStatus::MissCommaOrSquareBracket, "[1");
        test_error!(ParseStatus::MissCommaOrSquareBracket, "[1}");
        test_error!(ParseStatus::MissCommaOrSquareBracket, "[1,2");
        test_error!(ParseStatus::MissCommaOrSquareBracket, "[1 2");
        test_error!(ParseStatus::MissCommaOrSquareBracket, "[ [  ]");
        test_error!(ParseStatus::InvalidValue, "[,]");
        test_error!(ParseStatus::InvalidValue, "[1,]");
        test_error!(ParseStatus::InvalidValue, "[\"a\", nul]");
    }

    /// 测试解析 json 中的对象
    #[test]
    fn test_parse_object() {
        let mut node = LeptValue::default();

        // 测试空对象
        let status = LeptParser::parse(&mut node, "{  }");
        assert_eq!(ParseStatus::Ok, status);
        // 测试复杂对象
        let status = LeptParser::parse(
            &mut node,
            " { 
                \"n\" : null , 
                \"f\" : false , 
                \"t\" : true , 
                \"i\" : 123 , 
                \"s\" : \"abc\", 
                \"a\" : [ 1, 2, 3 ],
                \"o\" : { \"1\" : 1, \"2\" : 2, \"3\" : 3 }
            } ",
        );
        assert_eq!(ParseStatus::Ok, status);

        // 测试报错
        let status = LeptParser::parse(&mut node, "{ 1 }");
        assert_eq!(ParseStatus::MissKey, status);
        let status = LeptParser::parse(&mut node, "{ \"a\" }");
        assert_eq!(ParseStatus::MissColon, status);
        let status = LeptParser::parse(&mut node, "{ \"a\": }");
        assert_eq!(ParseStatus::InvalidValue, status);
        let status = LeptParser::parse(&mut node, "{ \"a\": 1");
        assert_eq!(ParseStatus::MissCommaOrCurlyBracket, status);
    }
}
