pub struct Embedding {
    text: String,
    vector: Vec<f32>,
}

impl Embedding {
    pub fn new(text: String, embedding: Vec<f32>) -> Embedding {
        Embedding {
            text,
            vector: embedding,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn vector(&self) -> &Vec<f32> {
        &self.vector
    }
}
