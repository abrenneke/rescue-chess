use crate::Position;
use std::collections::HashMap;

/// Represents a parsed EPD operation
#[derive(Debug, Clone, PartialEq)]
pub enum EpdOperand {
    String(String),
    SanMove(String),
    Integer(i32),
    Float(f64),
    Unsigned(u32),
}

/// Represents the Extended Position Description (EPD) format
#[derive(Debug, Clone)]
pub struct ExtendedPosition {
    /// The underlying chess position
    pub position: Position,
    /// Operations included in the EPD notation (opcode -> operands)
    pub operations: HashMap<String, Vec<EpdOperand>>,
}

impl ExtendedPosition {
    /// Creates a new ExtendedPosition from a Position with no operations
    pub fn new(position: Position) -> Self {
        ExtendedPosition {
            position,
            operations: HashMap::new(),
        }
    }

    /// Parses an EPD string into an ExtendedPosition
    pub fn parse_from_epd(notation: &str) -> Result<Self, anyhow::Error> {
        // First, find the four mandatory FEN fields
        let mut fields_iter = notation.split_whitespace();
        let mut fen_parts = Vec::new();

        // Collect the first 4 mandatory fields
        for _ in 0..4 {
            if let Some(field) = fields_iter.next() {
                fen_parts.push(field);
            } else {
                return Err(anyhow::anyhow!(
                    "EPD must contain the four mandatory position fields"
                ));
            }
        }

        // Construct FEN string with default halfmove and fullmove numbers
        let fen = format!("{} 0 1", fen_parts.join(" "));

        // Parse the base position using existing FEN parser
        let position = Position::parse_from_fen(&fen)?;
        let mut epd = ExtendedPosition::new(position);

        // The rest of the string contains operations
        let operations_str = fields_iter.collect::<Vec<_>>().join(" ");

        // Split operations by semicolons
        for op_str in operations_str.split(';') {
            let op_str = op_str.trim();
            if op_str.is_empty() {
                continue;
            }

            // Split into opcode and operands
            let mut op_parts = op_str.split_whitespace();
            let opcode = op_parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid operation format"))?
                .to_string();

            let mut operands = Vec::new();

            // Parse each operand
            while let Some(operand) = op_parts.next() {
                let parsed_operand = if operand.starts_with('"') {
                    // String operand - need to handle multi-word strings
                    let mut string_value = operand.to_string();
                    while !string_value.ends_with('"') {
                        if let Some(next_part) = op_parts.next() {
                            string_value.push(' ');
                            string_value.push_str(next_part);
                        } else {
                            break;
                        }
                    }
                    // Remove surrounding quotes
                    string_value = string_value.trim_matches('"').to_string();
                    EpdOperand::String(string_value)
                } else if operand.contains('.') {
                    // Float operand
                    EpdOperand::Float(operand.parse()?)
                } else if operand.starts_with('+') || operand.starts_with('-') {
                    // Integer operand
                    EpdOperand::Integer(operand.parse()?)
                } else if operand.chars().all(|c| c.is_ascii_digit()) {
                    // Unsigned operand
                    EpdOperand::Unsigned(operand.parse()?)
                } else {
                    // SAN move operand
                    EpdOperand::SanMove(operand.to_string())
                };

                operands.push(parsed_operand);
            }

            epd.operations.insert(opcode, operands);
        }

        Ok(epd)
    }

    /// Converts the EPD position back to string notation
    pub fn to_epd(&self) -> String {
        let mut epd = String::new();

        // Add the position part using existing FEN conversion
        let fen = self.position.to_fen();
        let fen_parts: Vec<_> = fen.split_whitespace().take(4).collect();
        epd.push_str(&fen_parts.join(" "));

        // Add operations
        for (opcode, operands) in &self.operations {
            epd.push(' ');
            epd.push_str(opcode);

            for operand in operands {
                epd.push(' ');
                match operand {
                    EpdOperand::String(s) => {
                        epd.push('"');
                        epd.push_str(s);
                        epd.push('"');
                    }
                    EpdOperand::SanMove(m) => epd.push_str(m),
                    EpdOperand::Integer(i) => epd.push_str(&i.to_string()),
                    EpdOperand::Float(f) => epd.push_str(&f.to_string()),
                    EpdOperand::Unsigned(u) => epd.push_str(&u.to_string()),
                }
            }
            epd.push(';');
        }

        epd
    }

    /// Gets the value of a specific operation
    pub fn get_operation(&self, opcode: &str) -> Option<&Vec<EpdOperand>> {
        self.operations.get(opcode)
    }

    /// Sets an operation with its operands
    pub fn set_operation(&mut self, opcode: String, operands: Vec<EpdOperand>) {
        self.operations.insert(opcode, operands);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_epd() {
        let epd = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - hmvc 0;";
        let pos = ExtendedPosition::parse_from_epd(epd).unwrap();

        assert_eq!(
            pos.get_operation("hmvc"),
            Some(&vec![EpdOperand::Unsigned(0)])
        );
    }

    #[test]
    fn test_parse_complex_epd() {
        let epd = "r1bqk2r/p1pp1ppp/2p2n2/8/1b2P3/2N5/PPP2PPP/R1BQKB1R w KQkq - bm Bd3; id \"Crafty Test Pos.28\"; c0 \"DB/GK Philadelphia 1996\";";
        let pos = ExtendedPosition::parse_from_epd(epd).unwrap();

        assert_eq!(
            pos.get_operation("bm"),
            Some(&vec![EpdOperand::SanMove("Bd3".to_string())])
        );
        assert_eq!(
            pos.get_operation("id"),
            Some(&vec![EpdOperand::String("Crafty Test Pos.28".to_string())])
        );
        assert_eq!(
            pos.get_operation("c0"),
            Some(&vec![EpdOperand::String(
                "DB/GK Philadelphia 1996".to_string()
            )])
        );
    }

    #[test]
    fn test_roundtrip() {
        let original = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - hmvc 0; fmvn 1;";
        let pos = ExtendedPosition::parse_from_epd(original).unwrap();
        let generated = pos.to_epd();

        // Parse both again to compare (to handle insignificant whitespace differences)
        let pos1 = ExtendedPosition::parse_from_epd(&original).unwrap();
        let pos2 = ExtendedPosition::parse_from_epd(&generated).unwrap();

        assert_eq!(pos1.operations, pos2.operations);
        // Could add more detailed position comparison if needed
    }
}
