# Suppositions

A property testing library for Rust.

Bringing some of the ideas from [Hypothesis](https://github.com/HypothesisWorks/hypothesis-python), [Theft](https://github.com/HypothesisWorks/hypothesis-python) and [Hedgehog](https://hedgehog.qa/).

I aim to bring in the compositional data generation from Hedgehog and Hypothesis and the [shrinking approach](http://hypothesis.works/articles/compositional-shrinking/) from Hypothesis.

## Compositional data generators

In regular quickcheck-alikes, you generally specify data generators per type. This is fine most of the time, but if you only want to check a subset of your inputs, then you end up having to tell the library to skip items needlessly.

Like hypothesis, we generate data by sampling an underlying stream of bytes; where we add the constraint that where the stream is "smaller" (either smaller values or shorter) the generated values should be similarly smaller.

This also means that the shrunk values from your first failing test will fulfil the constraints imposed by your generators.

## Integrated shrinking.

This essentially falls out of the above. Because generation is done from an underlying format; we don't need to re-implement shrinking for each individual type.