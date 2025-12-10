pub struct Scanner {

}

impl Scanner {
    pub fn new(_content: String) -> Self {
        Scanner {

        }
    }

    pub fn scan_tokens(&self) -> Vec<String> {
        vec!["TEST_TOKEN".to_string()]
    }
}