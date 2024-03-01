# fftools
Force field fitting analysis utilities

## ffblame
Reads a CSV of [QCArchive][qcarchive] ID, value pairs; the original dataset JSON
file; and an [OpenFF force field][openff], and prints the average value
associated with each parameter to stdout. For example, with a DDE CSV file like:

``` csv
,difference
36975451,2.864108483288476
36975452,-1.4599060830660164
36975453,0.7420562822326104
36975454,0.9236563755133034
36975455,2.4673481402655852
36975456,-3.397400659203896
36991787,-0.3981073050663806
```

the output might look something like

``` text
t100,0.66524352
t153,-0.08753583
t154,-0.50895293
t27,0.18993057
t141a,-0.05254415
t2,0.03755778
```

with an invocation like

``` shell
ffblame dde.csv industry.json openff-2.1.0.offxml
```

<!-- References -->
[qcarchive]: https://qcarchive.molssi.org/
[openff]: https://openforcefield.org/force-fields/force-fields/
