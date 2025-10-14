# DN2D: Datalog with Negation to Differencial Dataflow

This project explores the translation of recursive Datalog programs supporting negation into a differential dataflow framework, providing efficient incremental updates in response to evolving datasets.

## Why do we implement `DN2D`?

Current computational models designed for processing dynamically evolving input data struggle to efficiently facilitate iterative queries, except in specific and limited scenarios.

This limitation poses challenges when attempting complex tasks like real-time social-graph analysis, which could significantly benefit analysts studying the behavior of services such as Twitter.

However, a notable exception is found in the form of differential dataflow, which is built upon the concept of differential computation. Within this computational model, tasks are executed incrementally, relying on changes in the input, thereby substantially reducing the computational effort required to produce new results.

This study seeks to translate recursive Datalog programs that support negation into a differential dataflow framework, enabling them to effectively respond to evolving input data, addressing a critical need in data processing.

## Learning Material:
* [Intro to DDlog](https://chasewilson.dev/blog/intro-to-ddlog/)
* [Incremental Static Analysis with Differential Datalog](papers/Incremental Static Analysis with Differential Datalog.pdf)

## Related Implementations:

1. https://github.com/vmware-archive/differential-datalog
2. https://github.com/TimelyDataflow/differential-dataflow
3. https://github.com/TimelyDataflow/timely-dataflow

## Research Papers:

1. **McSherry et al**, Differential dataflow
2. **Alvarez-Picallo et al**, Fixing Incremental Computation: Derivatives of Fixpoints, and the Recursive Semantics of Datalog
3. **Behrend**, Efficient Computation of the Well-Founded Model Using Update Propagation
