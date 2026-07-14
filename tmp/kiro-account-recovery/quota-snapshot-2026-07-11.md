# Kiro account recovery quota snapshot

Snapshot time: 2026-07-11 00:36 +08:00

The values below are the last successful balance snapshots captured before
starting credential recovery. Passwords, 2FA codes, and tokens are never stored
in this file.

| Existing account | Auth state | Current usage | Usage limit | Remaining |
| --- | --- | ---: | ---: | ---: |
| KitWilliam | IDC refresh token invalid; balance cache unavailable | unknown | 1000 (subscription metadata only) | unknown |
| laohan_MarleyMacdo | IDC refresh token invalid; balance cache unavailable | unknown | 1000 (subscription metadata only) | unknown |
| laohan3 | GitHub social; auth_401 | 316.89 | 1000 | 683.11 |
| laohan4 | GitHub social; auth_401 | 309.16 | 1000 | 690.84 |
| laohan5 | GitHub social; auth_401 | 736.09 | 1000 | 263.91 |
| laohan6 | GitHub social; auth_401 | 285.11 | 1000 | 714.89 |
| tzpatric | GitHub social; auth_401 | 299.46 | 1000 | 700.54 |
| wqalerian | GitHub social; auth_401 | 321.36 | 1000 | 678.64 |

## Recovery observations

| GitHub login | Recovered email | Current usage | Usage limit | Remaining | Existing account match |
| --- | --- | ---: | ---: | ---: | --- |
| linkueisz3 | Adrian.Carlisle@asu.edu | 0 | 1000 | 1000 | No upstream user-id match; quota excludes the six social accounts above, but cannot distinguish KitWilliam from laohan_MarleyMacdo because both old balance caches are unavailable. Safely retained as `kiro-linkueisz3-github-social` pending later elimination. |
| linkueisz2 | Nolan.arlisle@asu.edu | 309.16 | 1000 | 690.84 | `laohan4` (exact upstream user-id and quota match; token and email restored) |
| linkueisz4 | Calebkl1lis@asu.edu | 285.11 | 1000 | 714.89 | `laohan6` (exact upstream user-id and quota match; token and email restored) |
| linkueisz5 | Calebkl2lis@asu.edu | 736.09 | 1000 | 263.91 | `laohan5` (exact upstream user-id and quota match; token and email restored) |
| linkueisz8 | Jinbo.Pan@asu.edu | 396.25 | 1000 | 603.75 | `laohan9` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz9 | BiHankiro2@asu.edu | 426.91 | 1000 | 573.09 | `linkuei_3_laohan7` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz10 | BiHankiro3@asu.edu | 622.34 | 1000 | 377.66 | `laohan10` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz11 | BiHankiro4@asu.edu | 305.30 | 1000 | 694.70 | `laohan11` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz12 | BiHankiro5@asu.edu | 605.24 | 1000 | 394.76 | `laohan12` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz13 | BiHankiro6@asu.edu | 387.12 | 1000 | 612.88 | `laohan13` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz14 | BiHankiro7@asu.edu | 372.32 | 1000 | 627.68 | `laohan14` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz15 | BiHankiro8@asu.edu | 648.35 | 1000 | 351.65 | `laohan15` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz16 | BiHankiro9@asu.edu | 418.70 | 1000 | 581.30 | `laohan16` (GitHub password rotated separately; exact upstream user-id match; token and email restored) |
| linkueisz17 | BiHankiro10@asu.edu | 546.58 | 1000 | 453.42 | `laohan17` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| linkueisz18 | BiHankiro13@asu.edu | 350.98 | 1000 | 649.02 | `laohan18` (exact upstream user-id match; existing account outside the original eight-account snapshot; token and email restored) |
| tzpatric | tzpatrick@utexas.edu | 299.46 | 1000 | 700.54 | `tzpatric` (exact upstream user-id and quota match; token and email restored) |
| wqalerian | wqalerian@utexas.edu | 321.36 | 1000 | 678.64 | `wqalerian` (exact upstream user-id and quota match; token and email restored) |
| oorcrofta | oorcrofta@utexas.edu | 338.48 | 1000 | 661.52 | `oorcrofta` (exact upstream user-id and quota match; token and email restored) |
| shfordsdf | shfordsdf@utexas.edu | 262.65 | 1000 | 737.35 | `shfordsdf` (exact upstream user-id and quota match; token and email restored) |
| ddggte | reuiiisa@uchicago.edu | 443.87 | 1000 | 556.13 | `ddggte` (exact upstream user-id and quota match; token and email restored; expired `dmit-us` binding replaced with `do-us-2`) |
| agnoliaqwd | agnolia@utexas.edu | 1000.00 | 1000 | 0.00 | `agnoliaqwd` (exact upstream user-id and quota match; token and email restored; current quota exhausted) |
| haedradw | haedra@utexas.edu | 324.33 | 1000 | 675.67 | `haedradw` (exact upstream user-id and quota match; token and email restored) |
| bbottadq | bbottadq@utexas.edu | 609.18 | 1000 | 390.82 | `bbottadq` (exact upstream user-id and quota match; token and email restored) |
| verharta | verharta@utexas.edu | 308.99 | 1000 | 691.01 | `verharta` (exact upstream user-id and quota match; token and email restored) |
| embrookew | embrooke@utexas.edu | 616.36 | 1000 | 383.64 | `embrookew` (exact upstream user-id and quota match; token and email restored) |
| unfieldgt | unfield@utexas.edu | 350.98 | 1000 | 649.02 | `unfieldgt` (exact upstream user-id and quota match; token and email restored) |
| elestineaw | elestineaw@utexas.edu | 495.24 | 1000 | 504.76 | `elestineaw` (exact requested GitHub login/account/email match; original cache had no upstream user-id; token and email restored) |
| allowayw | allowayw@utexas.edu | 464.61 | 1000 | 535.39 | `allowayw` (exact upstream user-id and quota match; token and email restored) |
| ghtingaleaw | ghtingaleaw@utexas.edu | 434.14 | 1000 | 565.86 | `ghtingaleaw` (exact upstream user-id and quota match; token and email restored) |
| orvalede | orvalede@utexas.edu | 670.25 | 1000 | 329.75 | `orvalede` (exact upstream user-id and quota match; token and email restored) |
| arlowew | arlowew@utexas.edu | 454.23 | 1000 | 545.77 | `arlowew` (exact upstream user-id and quota match; token and email restored) |
| deoqew | deoqew@utexas.edu | 1000.00 | 1000 | 0.00 | `deoqew` (exact upstream user-id and quota match; token and email restored; current quota exhausted) |
| tellanfs | tellanfs@utexas.edu | 378.06 | 1000 | 621.94 | `tellanfs` (exact upstream user-id and quota match; token and email restored) |
| ghtingalew | ghtingalew@utexas.edu | 374.56 | 1000 | 625.44 | `ghtingalew` (exact upstream user-id and quota match; token and email restored) |
| Christiansenx | Christiansen@asu.edu | 627.44 | 1000 | 372.56 | `laohan19` (exact upstream user-id and quota match; token and email restored) |
| Thompsonx | Thompsonx@asu.edu | 379.80 | 1000 | 620.20 | `laohan20` (exact upstream user-id and quota match; token and email restored) |
| Julianazzzx | Julianaz@asu.edu | 334.66 | 1000 | 665.34 | `laohan21` (exact upstream user-id and quota match; token and email restored) |
| Leslieex | Leslieex@asu.edu | 506.95 | 1000 | 493.05 | `laohan22` (exact upstream user-id and quota match; token and email restored) |
| Yazminz | Yazmin@asu.edu | 468.84 | 1000 | 531.16 | `laohan23` (exact upstream user-id and quota match; token and email restored) |
| Hartmannxz | Hartmannx@asu.edu | 460.12 | 1000 | 539.88 | `laohan24` (exact upstream user-id and quota match; token and email restored) |
| Carmellaz | Carmellaz@asu.edu | 509.76 | 1000 | 490.24 | `laohan25` (exact upstream user-id and quota match; token and email restored) |
| Jacklynzz | Jacklynz@asu.edu | 492.74 | 1000 | 507.26 | `laohan26` (exact upstream user-id match; token and email restored) |
| Bradtkez | Bradtkez@asu.edu | 349.81 | 1000 | 650.19 | `laohan27` (exact upstream user-id and quota match; token and email restored) |
| Naderxxz | Naderxxz@asu.edu | 398.43 | 1000 | 601.57 | `laohan28` (exact upstream user-id and quota match; token and email restored) |
| deckoww | deckoww@asu.edu | 521.20 | 1000 | 478.80 | `laohan29` (exact upstream user-id match; token and email restored) |
| kertzmann1 | kertzmann@asu.edu | 419.93 | 1000 | 580.07 | `laohan30` (exact upstream user-id and quota match; token and email restored) |
| sschiller1 | sschiller@asu.edu | 552.05 | 1000 | 447.95 | `laohan31` (exact upstream user-id and quota match; token and email restored) |
| ddenesikk | ddenesikk@asu.edu | 380.14 | 1000 | 619.86 | `laohan32` (exact upstream user-id match; token and email restored) |
| Dooleyyy | Dooleyyy@asu.edu | 406.52 | 1000 | 593.48 | `laohan33` (exact upstream user-id and quota match; token and email restored) |
