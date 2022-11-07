use std::f64;


#[allow(dead_code)]
pub fn df_transform(data: &[(f64, f64)]) -> Vec<(f64, f64)>
{
    let len = data.len();

    (0..len).map(|k|
    {
        let len = len as f64;
        let k = k as f64;

        data.iter().enumerate().fold((0.0, 0.0),
            |(out_real, out_imaginary), (index, (real, imaginary))|
            {
                let index = index as f64;

                let p = (2.0 * f64::consts::PI)/len * k * index;

                let left = p.cos();
                let right = p.sin();

                let final_real = real*left + imaginary*right;
                let final_imaginary = left*imaginary - real*right;

                (out_real + final_real, out_imaginary + final_imaginary)
            })
    }).collect()
}

#[allow(dead_code)]
pub fn inverse_df_transform(data: &[(f64, f64)]) -> Vec<(f64, f64)>
{
    let len = data.len();
    let ratio = 1.0/len as f64;

    (0..len).map(|n|
    {
        let len = len as f64;
        let n = n as f64;

        let pair = data.iter().enumerate().fold((0.0, 0.0),
            |(out_real, out_imaginary), (index, (real, imaginary))|
            {
                let index = index as f64;

                let x = (2.0 * f64::consts::PI)/len * index * n;

                let final_real = real * x.cos() - imaginary * x.sin();
                let final_imaginary = real * x.sin() + imaginary * x.cos();

                (out_real + final_real, out_imaginary + final_imaginary)
            });

        (pair.0*ratio, pair.1*ratio)
    }).collect()
}