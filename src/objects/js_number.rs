use crate::well_known as wk;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JsNumber {
    value: f64,
}

impl JsNumber {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
    
    pub fn from_i32(value: i32) -> Self {
        Self {
            value: value as f64,
        }
    }
    
    pub fn from_i64(value: i64) -> Self {
        Self {
            value: value as f64,
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        s.parse().ok().map(Self::new)
    }
    
    pub fn value(&self) -> f64 {
        self.value
    }
    
    pub fn is_nan(&self) -> bool {
        self.value.is_nan()
    }
    
    pub fn is_infinite(&self) -> bool {
        self.value.is_infinite()
    }
    
    pub fn is_finite(&self) -> bool {
        self.value.is_finite()
    }
    
    pub fn is_integer(&self) -> bool {
        self.value.fract() == 0.0
    }
    
    pub fn is_safe_integer(&self) -> bool {
        self.is_integer() && self.value.abs() <= 2_i64.pow(53) as f64
    }
    
    pub fn abs(&self) -> Self {
        Self::new(self.value.abs())
    }
    
    pub fn ceil(&self) -> Self {
        Self::new(self.value.ceil())
    }
    
    pub fn floor(&self) -> Self {
        Self::new(self.value.floor())
    }
    
    pub fn round(&self) -> Self {
        Self::new(self.value.round())
    }
    
    pub fn trunc(&self) -> Self {
        Self::new(self.value.trunc())
    }
    
    pub fn sign(&self) -> Self {
        if self.value > 0.0 {
            Self::new(1.0)
        } else if self.value < 0.0 {
            Self::new(-1.0)
        } else {
            Self::new(0.0)
        }
    }
    
    pub fn pow(&self, exponent: &JsNumber) -> Self {
        Self::new(self.value.powf(exponent.value))
    }
    
    pub fn sqrt(&self) -> Self {
        Self::new(self.value.sqrt())
    }
    
    pub fn cbrt(&self) -> Self {
        Self::new(self.value.cbrt())
    }
    
    pub fn log(&self) -> Self {
        Self::new(self.value.ln())
    }
    
    pub fn log2(&self) -> Self {
        Self::new(self.value.log2())
    }
    
    pub fn log10(&self) -> Self {
        Self::new(self.value.log10())
    }
    
    pub fn max_value() -> Self {
        Self::new(f64::MAX)
    }
    
    pub fn min_value() -> Self {
        Self::new(f64::MIN)
    }
    
    pub fn epsilon() -> Self {
        Self::new(f64::EPSILON)
    }
    
    pub fn negative_infinity() -> Self {
        Self::new(f64::NEG_INFINITY)
    }
    
    pub fn positive_infinity() -> Self {
        Self::new(f64::INFINITY)
    }
    
    pub fn nan() -> Self {
        Self::new(f64::NAN)
    }
    
    pub fn to_string(&self) -> String {
        if self.is_nan() {
            wk::NAN.to_string()
        } else if self.is_infinite() {
            if self.value > 0.0 {
                wk::INFINITY.to_string()
            } else {
                format!("-{}", wk::INFINITY)
            }
        } else if self.value == 0.0 {
            "0".to_string()
        } else {
            format!("{}", self.value)
        }
    }
    
    pub fn to_fixed(&self, digits: u32) -> String {
        format!("{:.1$}", self.value, digits as usize)
    }
    
    pub fn to_exponential(&self, digits: u32) -> String {
        format!("{:.1$e}", self.value, digits as usize)
    }
    
    pub fn to_precision(&self, precision: u32) -> String {
        format!("{:.1$}", self.value, precision as usize)
    }
}

impl std::fmt::Display for JsNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<f64> for JsNumber {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<i32> for JsNumber {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<i64> for JsNumber {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsBigInt {
    value: i128,
}

impl JsBigInt {
    pub fn new(value: i128) -> Self {
        Self { value }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim_end_matches('n');
        s.parse().ok().map(Self::new)
    }
    
    pub fn value(&self) -> i128 {
        self.value
    }
    
    pub fn add(&self, other: &JsBigInt) -> Self {
        Self::new(self.value + other.value)
    }
    
    pub fn sub(&self, other: &JsBigInt) -> Self {
        Self::new(self.value - other.value)
    }
    
    pub fn mul(&self, other: &JsBigInt) -> Self {
        Self::new(self.value * other.value)
    }
    
    pub fn div(&self, other: &JsBigInt) -> Option<Self> {
        if other.value == 0 {
            None
        } else {
            Some(Self::new(self.value / other.value))
        }
    }
    
    pub fn rem(&self, other: &JsBigInt) -> Option<Self> {
        if other.value == 0 {
            None
        } else {
            Some(Self::new(self.value % other.value))
        }
    }
    
    pub fn neg(&self) -> Self {
        Self::new(-self.value)
    }
    
    pub fn abs(&self) -> Self {
        Self::new(self.value.abs())
    }
    
    pub fn to_string(&self) -> String {
        format!("{}n", self.value)
    }
}

impl std::fmt::Display for JsBigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}n", self.value)
    }
}

impl From<i128> for JsBigInt {
    fn from(value: i128) -> Self {
        Self::new(value)
    }
}
