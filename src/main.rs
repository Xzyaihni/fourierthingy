use std::{
    path::Path,
    env,
    process
};

use rand::Rng;

use image::GenericImageView;

#[allow(unused_imports)]
use df_transform::*;

use ff_transform::*;

mod df_transform;
mod ff_transform;


fn randomly_shift_wave(waves: &mut [(f64, f64)], config: &Config)
{
    let mut rng = rand::thread_rng();

    let index = if config.wave.is_none()
    {
        rng.gen_range(0..waves.len())
    } else
    {
        let wave = config.wave.unwrap();
        if wave >= waves.len()
        {
            panic!("wave index cannot be bigger than input");
        }

        wave
    };

    waves[index].0 += config.strength;
    waves[index].1 += config.strength;
}

fn encode_reals(mut data: impl Iterator<Item=impl Into<f64>>) -> (Vec<(f64, f64)>, bool)
{
    let mut out_data = Vec::new();
    let mut padded = false;

    while let Some(v) = data.next()
    {
        let second = data.next().map(|b| b.into()).unwrap_or_else(||
        {
            padded = true;
            0.0
        });
        let pair = (v.into(), second);

        out_data.push(pair);
    }

    (out_data, padded)
}

fn decode_reals(data: impl Iterator<Item=(f64, f64)>, padded: bool) -> Vec<f64>
{
    let mut out_data: Vec<f64> = data.flat_map(|(r, j)|
    {
        [r, j]
    }).collect();

    if padded
    {
        //could use the npm package is-odd here
        out_data.pop();
    }

    out_data
}

fn buggify_text(text: &str, config: &Config) -> String
{
    if text.is_empty()
    {
        return String::new();
    }

    let (mut data_points, padded) = encode_reals(text.chars().map(|c| c as u32));
    let original_len = data_points.len();

    let mut waves = ff_transform(&mut data_points);

    randomly_shift_wave(&mut waves, config);

    decode_reals(inverse_ff_transform(&mut waves, original_len).into_iter(), padded)
        .into_iter().filter_map(|v| char::from_u32(v.round() as u32)).collect()
}

fn buggify_image(input_path: &str, output_path: &str, config: &Config)
{
    if !Path::new(input_path).exists()
    {
        eprintln!("{input_path} not found");
        return;
    }

    let img = image::open(input_path).unwrap();

    let size = img.dimensions();

    let (mut data_points, padded) = encode_reals(img.into_bytes().into_iter());
    let original_len = data_points.len();

    let waves = ff_transform(&mut data_points);
    //randomly_shift_wave(&mut inner_waves, config);

    let len = waves.len();
    let mut waves = waves.into_iter().enumerate().map(|(i, v)|
    {
        let ratio = i as f64 / (len-1) as f64;
        if ratio<config.strength
        {
            (0.0, 0.0)
        } else
        {
            v
        }
    }).collect::<Vec<(f64, f64)>>();

    image::save_buffer(
        output_path,
        &decode_reals(inverse_ff_transform(&mut waves, original_len).into_iter(), padded)
            .into_iter().map(|v| v as u8).collect::<Vec<u8>>(),
        size.0,
        size.1,
        image::ColorType::Rgb8).unwrap();
}

enum BuggifyMode
{
    Text,
    Image
}

struct Config
{
    mode: BuggifyMode,
    input: String,
    strength: f64,
    wave: Option<usize>
}

impl Config
{
    pub fn parse(args: impl Iterator<Item=String>) -> Result<Self, String>
    {
        let mut mode = None;
        let mut strength = None;
        let mut wave = None;

        let mut input = None;

        let mut args = args.peekable();
        while let Some(arg) = args.next()
        {
            if args.peek().is_none()
            {
                input = Some(arg);
                break;
            }

            match arg.as_str()
            {
                "-m" | "--mode" =>
                {
                    let value = args.next().ok_or_else(|| "-m must have a value".to_string())?;
                    mode = Some(match value.to_lowercase().as_str()
                    {
                        "text" => Ok(BuggifyMode::Text),
                        "image" => Ok(BuggifyMode::Image),
                        x => Err(format!("{x} is not a valid mode"))
                    }?);
                },
                "-s" | "--strength" =>
                {
                    let value = args.next().ok_or_else(|| "-s must have a value".to_string())?;
                    strength = Some(value.parse().map_err(|err| format!("{err} ({value})"))?);
                },
                "-w" | "--wave" =>
                {
                    let value = args.next().ok_or_else(|| "-w must have a value".to_string())?;
                    wave = Some(value.parse().map_err(|err| format!("{err} ({value})"))?);
                },
                x => return Err(x.to_string())
            }
        }

        let mode = mode.ok_or_else(|| "-m option is mandatory".to_string())?;
        let strength = strength.ok_or_else(|| "-s option is mandatory".to_string())?;

        let input = input.ok_or("argument for input not found")?;

        Ok(Config{mode, input, strength, wave})
    }

    pub fn help_message() -> !
    {
        eprintln!("usage: {} [args] input", env::args().next().unwrap());
        eprintln!("args:");
        eprintln!("    -m, --mode           what to buggify (text, image)");
        eprintln!("    -s, --strength       strength of buggifying");
        eprintln!("    -w, --wave           wave to modify (default random)");
        process::exit(1);
    }
}

fn main()
{
    let config = Config::parse(env::args().skip(1)).unwrap_or_else(|err|
    {
        eprintln!("invalid arguments: {err}");
        Config::help_message()
    });

    match config.mode
    {
        BuggifyMode::Image => buggify_image(&config.input, "buggified.png", &config),
        BuggifyMode::Text => println!("{}", buggify_text(&config.input, &config))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn test_buggify(text: &str)
    {
        let out = buggify_text(text, 0);

        println!("in: {:?}", text.chars().map(|c| c as u32).collect::<Vec<u32>>());
        println!("out: {:?}", out.chars().map(|c| c as u32).collect::<Vec<u32>>());
        assert_eq!(out, text);
    }

    fn random_amount_test(amount: usize)
    {
        let mut rng = rand::thread_rng();

        let text = &(0..amount).map(|_| rng.gen_range(32..127u8) as char).collect::<String>();
        test_buggify(text);
    }

    #[test]
    fn one_random_fourier_text()
    {
        random_amount_test(1);
    }

    #[test]
    fn two_random_fourier_text()
    {
        random_amount_test(2);
    }

    #[test]
    fn four_random_fourier_text()
    {
        random_amount_test(4);
    }

    #[test]
    fn eight_random_fourier_text()
    {
        random_amount_test(8);
    }

    #[test]
    fn small_random_fourier_text()
    {
        random_amount_test(10);
    }

    #[test]
    fn large_random_fourier_text()
    {
        random_amount_test(100);
    }
}