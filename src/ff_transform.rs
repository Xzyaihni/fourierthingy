use std::f64;


fn pad_to_nearest_power(data: &mut Vec<(f64, f64)>)
{
    let len = data.len();

    let mut closest_length = 1;
    if (len & (len - 1))!=0
    {
        while closest_length<len
        {
            closest_length <<= 1;
        }

        data.resize(closest_length, (0.0, 0.0));
    }
}

pub fn ff_transform(data: &mut [(f64, f64)]) -> Vec<(f64, f64)>
{
    let mut data_vec = data.to_vec();
    pad_to_nearest_power(&mut data_vec);

    fft::<false>(&data_vec)
}

pub fn inverse_ff_transform(data: &mut [(f64, f64)], original_len: usize) -> Vec<(f64, f64)>
{
    let len = data.len();
    if (len & (len - 1))!=0
    {
        panic!("length should be a power of 2, instead its {}", len);
    }

    let mut data_vec = fft::<true>(data).into_iter().map(|v|
    {
        let n = len as f64;
        (v.0/n, v.1/n)
    }).collect::<Vec<(f64, f64)>>();
    data_vec.truncate(original_len);

    data_vec
}

fn fft<const INVERSE: bool>(data: &[(f64, f64)]) -> Vec<(f64, f64)>
{
    let len = data.len();
    if len==1
    {
        return data.to_vec();
    }

    let x = (if INVERSE{-2.0}else{2.0} * f64::consts::PI)/len as f64;

    let a = x.cos();
    let b = x.sin();

    let even_half = fft::<INVERSE>(&data.iter().cloned().step_by(2)
        .collect::<Vec<(f64, f64)>>());

    let odd_half = fft::<INVERSE>(&data.iter().cloned().skip(1).step_by(2)
        .collect::<Vec<(f64, f64)>>());


    let mut out = vec![(0.0, 0.0);len];
    for i in 0..len/2
    {
        let r = (a*a + b*b).sqrt().powi(i as i32);
        let angle = i as f64 * (b/a).atan();

        let pow_real = r * angle.cos();
        let pow_imaginary = r * angle.sin();

        let odd_real = pow_real*odd_half[i].0 - pow_imaginary*odd_half[i].1;
        let odd_imaginary = pow_real*odd_half[i].1 + pow_imaginary*odd_half[i].0;

        out[i].0 = even_half[i].0 + odd_real;
        out[i].1 = even_half[i].1 + odd_imaginary;

        out[i+len/2].0 = even_half[i].0 - odd_real;
        out[i+len/2].1 = even_half[i].1 - odd_imaginary;
    }

    out
}