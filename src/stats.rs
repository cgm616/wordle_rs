use std::{fmt::Debug, iter::Sum};

use num_traits::Float;
use statrs::distribution::{ContinuousCDF, StudentsT};

use crate::{Result, WordleError};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Tails {
    One,
    Two,
}

impl Tails {
    fn factor<N: Float>(&self) -> N {
        match self {
            Self::One => N::from(1_f32).unwrap(),
            Self::Two => N::from(2_f32).unwrap(),
        }
    }
}

// struct Sample<'b, N: 'b + Float + Sum, T: IntoIterator<Item = &'b N>>(T);

struct Sample<N: Float> {
    mean: N,
    len: N,
    var: N,
}

// impl<'b, N: 'b + Float + Sum, T: Clone + IntoIterator<Item = &'b N>> Sample<'b, N, T> {
//     fn mean(&self) -> N {
//
//     }
// }

impl<N: Float + Sum> Sample<N> {
    fn new<T: IntoIterator<Item = N> + Clone>(sample: T) -> Self {
        let (len, sum) =
            sample
                .clone()
                .into_iter()
                .fold((0_u32, N::from(0_f32).unwrap()), |acc, next| {
                    let count = acc.0 + 1_u32;
                    let sum = acc.1 + next;

                    (count, sum)
                });

        let mean = sum / N::from(len).unwrap();

        let var =
            sample.into_iter().map(|n| (n - mean).powi(2)).sum::<N>() / N::from(len - 1).unwrap();

        Sample {
            mean,
            len: N::from(len).unwrap(),
            var,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct WelchsT<N: Float> {
    /// The p-value of the test, which is the probability accepting the results
    /// of the test is an error because the null hypothesis is in fact true.
    pub(crate) p: N,

    /// The maximum allowed p-value.
    pub(crate) alpha: N,

    /// The "tails" of the test.
    pub(crate) tails: Tails,
}

impl<N: Float + Sum + Into<f64>> WelchsT<N> {
    /// Runs the test on two samples.
    ///
    /// # Panics
    ///
    /// `alpha` must be in (0, 1).
    pub(crate) fn two_sample<
        T: IntoIterator<Item = N> + Clone,
        V: IntoIterator<Item = N> + Clone,
    >(
        a: T,
        b: V,
        alpha: N,
        tails: Tails,
    ) -> Result<Self> {
        assert!(alpha > N::from(0_f32).unwrap() && alpha < N::from(1_f32).unwrap());
        // To run the t-test, we need to calculate the test statistic and then
        // compare it against the T distribution.

        let a = Sample::new(a);
        let b = Sample::new(b);

        if a.mean.into().abs() < f64::EPSILON || b.mean.into().abs() < f64::EPSILON {
            return Err(WordleError::Stats);
        }

        // Uses equations from https://statisticaloddsandends.wordpress.com/2020/07/03/welchs-t-test-and-the-welch-satterthwaite-equation/.

        // Calculate t-statistic.
        let t = (a.mean - b.mean).abs() / ((a.var / a.len) + (b.var / b.len)).sqrt();

        // Calculate degrees of freedom.
        let deg = ((a.var / a.len) + (b.var / b.len)).powi(2)
            / ((a.var.powi(2) / (a.len.powi(2) * (a.len - N::from(1_u32).unwrap())))
                + (b.var.powi(2) / (b.len.powi(2) * (b.len - N::from(1_u32).unwrap()))));

        let dist = StudentsT::new(0.0, 1.0, deg.into()).unwrap();

        let p = N::from(dist.cdf((-t).into())).unwrap() * tails.factor::<N>();

        Ok(Self { p, alpha, tails })
    }

    #[allow(dead_code)]
    pub(crate) fn is_significant(&self) -> bool {
        self.p < self.alpha
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use pyo3::{
        prelude::*,
        types::{IntoPyDict, PyDict},
    };
    use rand::distributions::Distribution;
    use statrs::{assert_almost_eq, distribution::Normal};

    use std::result::Result as StdResult;

    use super::*;

    proptest! {
        #[test]
        fn matches_python(
            f_bar in 0.5_f64..10.0,
            s_bar in 0.5_f64..10.0,
            f_std in 0.1_f64..1.0,
            s_std in 0.1_f64..1.0,
            samples in 10..1000
        ) {
            const ALPHA: f64 = 0.05;

            let mut rng = rand::thread_rng();

            let f_dist = Normal::new(f_bar, f_std).unwrap();
            let s_dist = Normal::new(s_bar, s_std).unwrap();

            let f_samples: Vec<f64> = (0..samples).map(|_| f_dist.sample(&mut rng)).collect();
            let s_samples: Vec<f64> = (0..samples).map(|_| s_dist.sample(&mut rng)).collect();

            let internal: WelchsT<f64> = WelchsT::two_sample(f_samples.iter().cloned(), s_samples.iter().cloned(), ALPHA, Tails::Two)?;

            let external: StdResult<f64, PyErr> = Python::with_gil(|py| {
                let locals = PyDict::new(py);
                py.run("from scipy import stats", None, Some(locals))?;
                let stats = locals.get_item("stats").unwrap();

                // let scipy = py.import("scipy")?;
                // let stats = scipy.import("stats")?;

                let f_samples = f_samples.to_object(py);
                let s_samples = s_samples.to_object(py);

                let args = (f_samples, s_samples);
                let kwargs = [("equal_var", false)];
                let res = stats.getattr("ttest_ind")?.call(args, Some(kwargs.into_py_dict(py)))?;
                res.getattr("pvalue")?.extract()
            });

            let external = match external {
                Ok(f) => f,
                Err(e) => panic!("Python encountered error: {}", e)
            };

            assert!((internal.p - external).abs() < 0.000001_f64);
        }
    }
}
