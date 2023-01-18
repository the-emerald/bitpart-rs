use nom::{
    character::complete::{self, newline, space1},
    combinator::verify,
    multi::{many1, separated_list0},
    number::complete::double,
    IResult,
};

/// The configuration for a particular test file. This is the first line in a SISAP test file.
#[derive(Debug, Clone, Copy)]
pub struct FileConfig {
    /// Dimension size for each vector
    pub dimensions: u64,
    /// Number of vectors in dataset
    pub lines: u64,
    /// Very mysterious, indeed
    pub mysterious: u64,
}

fn vector(input: &str) -> IResult<&str, Vec<f64>> {
    let (input, vector) = separated_list0(space1, double)(input)?;
    let (input, _) = newline(input)?;
    Ok((input, vector))
}

fn config(input: &str) -> IResult<&str, FileConfig> {
    let (input, dimensions) = complete::u64(input)?;
    let (input, _) = space1(input)?;

    let (input, lines) = complete::u64(input)?;
    let (input, _) = space1(input)?;

    let (input, mysterious) = complete::u64(input)?;

    Ok((
        input,
        FileConfig {
            dimensions,
            lines,
            mysterious,
        },
    ))
}

pub fn parse(input: &str) -> IResult<&str, (FileConfig, Vec<Vec<f64>>)> {
    //todo use const generics?
    let (input, file_config) = config(input)?;
    let (input, _) = newline(input)?;
    let (input, vectors) = verify(
        many1(verify(vector, |v: &Vec<f64>| {
            v.len() == file_config.dimensions as usize
        })),
        |v: &Vec<Vec<f64>>| v.len() == file_config.lines as usize,
    )(input)?;

    Ok((input, (file_config, vectors)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NASA: &str = include_str!("nasa/nasa.ascii");

    #[test]
    fn test_nasa() {
        let (config, vectors) = parse(NASA).unwrap().1;

        assert_eq!(config.dimensions, 20);
        assert_eq!(config.lines, 40150);
        assert!(vectors
            .iter()
            .all(|v| v.len() == config.dimensions as usize));
        assert_eq!(config.lines as usize, vectors.len());
    }
}
