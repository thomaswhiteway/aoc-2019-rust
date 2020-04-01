use std::io::stdin;
use std::str;
use std::fmt;
use std::char;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Pixel {
    Black,
    White,
    Transparent,
}

impl Pixel {
    fn parse(c: char) -> Result<Self, String> {
        use Pixel::*;
        match c {
            '0' => Ok(Black),
            '1' => Ok(White),
            '2' => Ok(Transparent),
            _ => Err(format!("Invalid pixel {}", c)),
        }
    }

    fn as_char(&self) -> char {
        use Pixel::*;
        match self {
            Black => char::from_u32(0x2588).unwrap(),
            _ => ' '
        }
    }
}

struct Layer {
    data: Box<[Pixel]>,
    width: usize,
    height: usize,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            writeln!(f, "{}", self.data[row*self.width..(row+1)*self.width].iter().map(Pixel::as_char).collect::<String>())?;
        }
        Ok(())
    }
}

impl Layer {
    fn new(data: Box<[Pixel]>, width: usize, height: usize) -> Self {
        Layer {
            data,
            width,
            height,
        }
    }

    fn merge(&self, layer: &Layer) -> Layer {
        let data: Vec<_> = self
            .data
            .iter()
            .zip(layer.data.iter())
            .map(|(top, bottom)| {
                if *top != Pixel::Transparent {
                    *top
                } else {
                    *bottom
                }
            })
            .collect();
        Layer::new(data.into_boxed_slice(), self.width, self.height)
    }
}

fn parse_layers<'a>(
    data: &'a str,
    width: usize,
    height: usize,
) -> impl Iterator<Item = Layer> + 'a {
    data.trim().as_bytes().chunks(width * height).map(move |bytes| {
        let data: Vec<_> = str::from_utf8(bytes)
            .unwrap()
            .chars()
            .map(Pixel::parse)
            .map(Result::unwrap)
            .collect();
        Layer::new(data.into_boxed_slice(), width, height)
    })
}

fn combine_layers<'a>(mut layers: impl Iterator<Item = Layer>) -> Layer {
    let first_layer = layers.next().unwrap();
    layers.fold(first_layer, |current, layer| current.merge(&layer))
}

fn main() {
    let mut data = String::new();
    stdin().read_line(&mut data).unwrap();

    let layers = parse_layers(&data, 25, 6);

    let layer = combine_layers(layers);

    println!("{}", layer);
}
