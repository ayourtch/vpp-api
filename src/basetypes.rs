pub enum basetypes{
    U8,
    I8,
    U16,
    I16, 
    U32, 
    I32, 
    U64, 
    I64, 
    F64, 
    BOOL, 
    STRING  
}

impl basetypes{
    fn basetypeSizes(&self) -> u8{
        match self {
            basetypes::U8 => 1,
            basetypes::I8 => 1, 
            basetypes::U16 => 2,
            basetypes::I16 => 2, 
            basetypes::U32 => 4, 
            basetypes::I32 => 4, 
            basetypes::U64 => 8, 
            basetypes::I64 => 8,
            basetypes::F64 => 8,  
            basetypes::BOOL => 1, 
            basetypes::STRING => 1
        }
    }
    fn rustTypes(&self) -> &str {
        match self {
            basetypes::U8 => "u8",
            basetypes::I8 => "i8", 
            basetypes::U16 => "u16",
            basetypes::I16 => "i16", 
            basetypes::U32 => "u32", 
            basetypes::I32 => "i32", 
            basetypes::U64 => "u64", 
            basetypes::I64 => "i64",
            basetypes::F64 => "f64",  
            basetypes::BOOL => "bool", 
            basetypes::STRING => "string"
        }
    }
    fn ctoSize(size: &str) -> basetypes{
        match size{
            "uint8" => basetypes::U8,
            "int8" => basetypes::I8, 
            "uint16" => basetypes::U16,
            "int16" => basetypes::I16, 
            "uint32" => basetypes::U32,
            "int32" => basetypes::I32,
            "uint64" => basetypes::U64,
            "int64" => basetypes::I64,
            "float64" => basetypes::F64,
            "bool" => basetypes::BOOL,
            "string" => basetypes::STRING,
            _ => basetypes::U8
        }
    }
    fn ctoSizeR(size: &str) -> basetypes{
        match size{
            "u8" => basetypes::U8,
            "i8" => basetypes::I8, 
            "u16" => basetypes::U16,
            "i16" => basetypes::I16, 
            "u32" => basetypes::U32,
            "i32" => basetypes::I32,
            "u64" => basetypes::U64,
            "i64" => basetypes::I64,
            "f64" => basetypes::F64,
            "bool" => basetypes::BOOL,
            "string" => basetypes::STRING,
            _ => basetypes::U8
        }
    }
    
}