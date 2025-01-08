use std::collections::HashMap;
use std::io::BufRead;
use crate::error::Error;
use crate::opcode::Opcode;

fn include_only_whitespace(s: &str) -> bool {
    for c in s.chars() {
        if !c.is_whitespace() {
            return false;
        }
    }

    true
}

pub fn split_code<T: BufRead>(mut code: T) -> Result<Vec<Vec<String>>, Error> {
    let mut ret = Vec::new();
    let mut buf = String::new();

    loop {
        buf.clear();
        match code.read_line(&mut buf) {
            Ok(0) => break,
            Err(err) => return Err(Error::IoError(err)),
            _ => (),
        }

        // Ignore a comment (after '#')
        let buf = buf.split('#').collect::<Vec<_>>()[0];

        // Skip a blank line
        if include_only_whitespace(buf) {
            continue;
        }

        let mut line = Vec::new();
        buf.split_whitespace().collect::<Vec<_>>()
            .into_iter()
            .for_each(|elem| {
                if elem.ends_with(':') {
                    // Colon located on a word's end is independent element
                    line.append(
                        &mut vec![
                            elem[..elem.len()-1].to_string(),
                            ":".to_string(),
                        ]
                    );
                } else {
                    line.push(elem.to_string());
                }
            });
        ret.push(line);
    }

    Ok(ret)
}

pub fn load_label(
    code: &Vec<Vec<String>>,
    label_table: &mut HashMap<String, usize>
) {
    label_table.clear();

    let mut line_num = 0;
    code.iter().for_each(|line| {
        if line.len() < 2 {
            line_num += 1;
            return;
        }
        if line[1] != ":" {
            line_num += 1;
            return;
        }

        label_table.insert(line[0].clone(), line_num);
    });
}

pub fn load_inst(
    code: &Vec<Vec<String>>,
    inst_memory: &mut Vec<Opcode>
) -> Result<(), Error> {
    inst_memory.clear();

    for line in code {
        if let Some(c) = line.get(1) {
            if c == ":" {
                continue;
            }
        }

        match Opcode::from_line(line) {
            Ok(op) => inst_memory.push(op),
            Err(err) => return Err(err),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn detect_whitespace() {
        assert!(include_only_whitespace(" \t \n "));
        assert!(include_only_whitespace("\r\n\r\n"));
        assert!(!include_only_whitespace("a\n"));
        assert!(!include_only_whitespace("これは空白ではありません"));
    }

    #[test]
    fn split_testcode() {
        let cursor = io::Cursor::new(
            b"L0:\n
              \tpushi 10\n
              \tpushi  5\n
              \tpushi\t7\n
              \t# Addition\n
              \tADD\n
              \tmul\n
              \tWr # Pop and write a value\n
              \twrln\n
              \tjp L0"
        );
        
        let tokens = split_code(cursor).unwrap();

        assert_eq!(
            tokens,
            vec![
                vec!["L0".to_string(), ":".to_string()],
                vec!["pushi".to_string(), "10".to_string()],
                vec!["pushi".to_string(), "5".to_string()],
                vec!["pushi".to_string(), "7".to_string()],
                vec!["ADD".to_string()],
                vec!["mul".to_string()],
                vec!["Wr".to_string()],
                vec!["wrln".to_string()],
                vec!["jp".to_string(), "L0".to_string()],
            ]
        );
    }

    #[test]
    fn give_labels_integers() {
        let code = vec![
            vec!["L0".to_string(), ":".to_string()],
            vec!["pushi".to_string(), "15".to_string()],
            vec!["jp".to_string(), "L1".to_string()],
            vec!["L1".to_string(), ":".to_string()],
            vec!["wr".to_string()],
            vec!["L2".to_string(), ":".to_string()],
            vec!["jp".to_string(), "L0".to_string()],
        ];
        let mut table = HashMap::new();

        load_label(&code, &mut table);

        assert_eq!(
            table,
            HashMap::from([
                ("L0".to_string(), 0),
                ("L1".to_string(), 2),
                ("L2".to_string(), 3),
            ])
        );
    }

    #[test]
    fn code_to_opcode() {
        let code = vec![
            vec!["L0".to_string(), ":".to_string()],
            vec!["pushi".to_string(), "10".to_string()],
            vec!["pushi".to_string(), "5".to_string()],
            vec!["pushi".to_string(), "7".to_string()],
            vec!["ADD".to_string()],
            vec!["mul".to_string()],
            vec!["Wr".to_string()],
            vec!["wrln".to_string()],
            vec!["jp".to_string(), "L0".to_string()],
        ];
        let mut memory = Vec::new();

        load_inst(&code, &mut memory).unwrap();

        assert_eq!(
            memory,
            vec![
                Opcode::Pushi(10),
                Opcode::Pushi(5),
                Opcode::Pushi(7),
                Opcode::Add,
                Opcode::Mul,
                Opcode::Wr,
                Opcode::Wrln,
                Opcode::Jp("L0".to_string())
            ]
        );
    }
}
