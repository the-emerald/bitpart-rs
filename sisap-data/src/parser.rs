use nom::{
    character::complete::{self, newline, space0, space1},
    combinator::verify,
    multi::{many1, separated_list0},
    number::complete::double,
    IResult,
};

/// The configuration of the dataset.
/// This corresponds to the first line in an `.ascii` file.
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
    let (input, _) = space0(input)?;
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

/// Parse a `.ascii` file.
/// Points are stored as a [Vec](mod@std::vec).
/// If the dimensionality of the dataset can be determined at compile-time,
/// [`parse_array`] can be used to remove one layer of indirection.
pub fn parse(input: &str) -> IResult<&str, (FileConfig, Vec<Vec<f64>>)> {
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

/// Parse an `.ascii` file.
/// Points are stored as an [array](core::array), but this requires the dimensionality
/// to be `const`. If this cannot be done at compile-time, use [`parse`].
///
/// # Panics
///
/// This function will panic if `N` does not match the dimension size specified in the file being parsed.
pub fn parse_array<const N: usize>(input: &str) -> IResult<&str, (FileConfig, Vec<[f64; N]>)> {
    let (input, file_config) = config(input)?;
    assert_eq!(N as u64, file_config.dimensions);

    let (input, _) = newline(input)?;
    let (input, vectors) = verify(
        many1(verify(vector, |v: &Vec<f64>| v.len() == N)),
        |v: &Vec<Vec<f64>>| v.len() == file_config.lines as usize,
    )(input)?;

    let vectors = vectors.into_iter().map(|x| x.try_into().unwrap()).collect();

    Ok((input, (file_config, vectors)))
}

#[cfg(test)]
mod tests {
    use crate::{colors::COLORS_DIMENSION, nasa::NASA_DIMENSION};

    use super::*;

    const NASA: &str = include_str!("nasa.ascii");
    const COLORS: &str = include_str!("colors.ascii");

    #[test]
    fn nasa() {
        let (config, vectors) = parse(NASA).unwrap().1;

        assert_eq!(config.dimensions, 20);
        assert_eq!(config.lines, 40150);
        assert!(vectors
            .iter()
            .all(|v| v.len() == config.dimensions as usize));
        assert_eq!(config.lines as usize, vectors.len());
    }

    #[test]
    fn nasa_const() {
        let (config, vectors) = parse_array::<NASA_DIMENSION>(NASA).unwrap().1;

        assert_eq!(config.dimensions, 20);
        assert_eq!(config.lines, 40150);
        assert!(vectors
            .iter()
            .all(|v| v.len() == config.dimensions as usize));
        assert_eq!(config.lines as usize, vectors.len());
    }

    #[test]
    fn colors() {
        let (config, vectors) = parse(COLORS).unwrap().1;

        assert_eq!(config.dimensions, 112);
        assert_eq!(config.lines, 112682);
        assert!(vectors
            .iter()
            .all(|v| v.len() == config.dimensions as usize));
        assert_eq!(config.lines as usize, vectors.len());
    }

    #[test]
    fn colors_const() {
        let (config, vectors) = parse_array::<COLORS_DIMENSION>(COLORS).unwrap().1;

        assert_eq!(config.dimensions, 112);
        assert_eq!(config.lines, 112682);
        assert!(vectors
            .iter()
            .all(|v| v.len() == config.dimensions as usize));
        assert_eq!(config.lines as usize, vectors.len());
    }

    #[test]
    #[should_panic]
    fn colors_const_wrong_dim() {
        let (_, _) = parse_array::<1111>(COLORS).unwrap().1;
    }
}
