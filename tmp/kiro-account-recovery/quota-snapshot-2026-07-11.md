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
| Bergstromx | Bergstrom@asu.edu | 721.49 | 1000 | 278.51 | `laohan34` (exact upstream user-id and quota match; token and email restored) |
| Martinexxxz | Martinex@asu.edu | 436.60 | 1000 | 563.40 | `laohan35` (exact upstream user-id match; token and email restored) |
| Kassulkee | Kassulke@asu.edu | 402.93 | 1000 | 597.07 | `laohan36` (exact upstream user-id and quota match; token and email restored) |
| Scottieeex | Scottieee@asu.edu | 839.43 | 1000 | 160.57 | `laohan37` (exact upstream user-id and quota match; token and email restored) |
| Francescoxz | Francesco@asu.edu | 993.14 | 1000 | 6.86 | `laohan38` (exact upstream user-id and quota match; token and email restored below the 10-credit importer threshold) |
| Prosacco1 | Prosacco@asu.edu | 885.25 | 1000 | 114.75 | `laohan39` (operator-confirmed sequential mapping; token and email restored) |
| Imeldaa1 | Imeldaa@asu.edu | 535.07 | 1000 | 464.93 | `laohan41` (operator-corrected mapping; token and email restored; mistaken `laohan40` import removed) |
| Hagenes12 | Hagenes@asu.edu | 546.07 | 1000 | 453.93 | `laohan42` (operator-confirmed sequential mapping; token and email restored) |
| Janessaa1 | Janessaa@asu.edu | 598.29 | 1000 | 401.71 | `laohan43` (operator-confirmed sequential mapping; token and email restored) |
| Kuhlmannn1 | Kuhlmannn@asu.edu | 586.64 | 1000 | 413.36 | `laohan44` (operator-confirmed sequential mapping; token and email restored) |
| Laurinez | Laurine@asu.edu | 524.67 | 1000 | 475.33 | `laohan45` (operator-confirmed sequential mapping; token and email restored) |
| Darlenexc | Darlenex@asu.edu | 477.59 | 1000 | 522.41 | `laohan46` (operator-confirmed sequential mapping; token and email restored) |
| Lehnerxz | Lehnerxz@asu.edu | 532.90 | 1000 | 467.10 | `laohan47` (operator-confirmed sequential mapping; token and email restored) |
| Lefflerxs | Lefflerxs@asu.edu | 507.02 | 1000 | 492.98 | `laohan48` (operator-confirmed sequential mapping; token and email restored) |
| Emoryyy1 | Emoryyy@asu.edu | 437.47 | 1000 | 562.53 | `laohan49` (operator-confirmed sequential mapping; token and email restored) |
| Deangeloo1 | Deangeloo@asu.edu | 580.08 | 1000 | 419.92 | `laohan50` (operator-confirmed sequential mapping; token and email restored) |
| Smithamm1 | Smithamm@asu.edu | 722.32 | 1000 | 277.68 | `laohan51` (operator-confirmed sequential mapping; token and email restored) |
| Nienoww | Nienoww@asu.edu | 1000.00 | 1000 | 0.00 | `laohan52` (operator-confirmed sequential mapping; token and email restored; current cycle exhausted) |
| Keelinggg | Keelinggg@asu.edu | 562.93 | 1000 | 437.07 | `laohan53` (operator-confirmed sequential mapping; token and email restored) |
| Reingerrr | Reingerrr@asu.edu | 735.24 | 1000 | 264.76 | `laohan54` (operator-confirmed sequential mapping; token and email restored) |
| Reynoldss12 | Reynoldss@asu.edu | 522.51 | 1000 | 477.49 | `laohan55` (operator-confirmed sequential mapping; token and email restored) |
| Kayleyu1 | Kayleyu@asu.edu | 521.82 | 1000 | 478.18 | `laohan56` (operator-confirmed sequential mapping after GitHub password and 2FA rotation; token and email restored) |
| Sengerr2 | Sengerr@asu.edu | 603.60 | 1000 | 396.40 | `laohan57` (operator-confirmed sequential mapping after GitHub password and 2FA rotation; token and email restored) |
| Schulist1 | Schulist@asu.edu | 991.94 | 1000 | 8.06 | `laohan58` (operator-confirmed sequential mapping after GitHub password and 2FA rotation; token and email restored below the 10-credit scheduling threshold) |
| Arnoldo123z | Arnoldo@asu.edu | 992.10 | 1000 | 7.90 | `laohan59` (operator-confirmed sequential mapping after GitHub password and 2FA rotation; token and email restored below the 10-credit scheduling threshold) |
| Kunzeez | Kunzeez@asu.edu | 671.25 | 1000 | 328.75 | `laohan60` (operator-confirmed sequential mapping after GitHub password and 2FA rotation; token and email restored) |
| Autumnn12 | Autumnn@asu.edu | 561.22 | 1000 | 438.78 | `laohan61` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Fritschh | Fritschh@asu.edu | 489.28 | 1000 | 510.72 | `laohan62` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Carminee123 | Carmine@asu.edu | 1000.00 | 1000 | 0.00 | `laohan63` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored; current cycle exhausted) |
| Hodkiewicz1 | Hodkiewicz@asu.edu | 932.82 | 1000 | 67.18 | `laohan64` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Clementinee1 | Clementinee@asu.edu | 852.57 | 1000 | 147.43 | `laohan65` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Kilback12 | Kilback@asu.edu | 592.06 | 1000 | 407.94 | `laohan66` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Maxweelll | Maxweelll@asu.edu | 994.05 | 1000 | 5.95 | `laohan67` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored below the 10-credit scheduling threshold) |
| Bernierzz | Bernierzz@asu.edu | 1000.00 | 1000 | 0.00 | `laohan68` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored; current cycle exhausted) |
| Marcellee1 | Marcellee@asu.edu | 995.57 | 1000 | 4.43 | `laohan69` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored below the 10-credit scheduling threshold) |
| Gulgowskiii | Gulgowski@asu.edu | 541.97 | 1000 | 458.03 | `laohan70` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Lindgrenx | Lindgrenx@asu.edu | 990.17 | 1000 | 9.83 | `laohan71` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored below the 10-credit scheduling threshold) |
| Kristoffercc | Kristofferc@asu.edu | 806.75 | 1000 | 193.25 | `laohan72` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Newellxxx | Newellxxx@asu.edu | 907.46 | 1000 | 92.54 | `laohan73` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Gutkowskizz | Gutkowskizz@asu.edu | 565.19 | 1000 | 434.81 | `laohan74` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Ahmedddz2 | Ahmedd@asu.edu | 547.25 | 1000 | 452.75 | `laohan77` (operator-confirmed sequence continued to the next existing auth_401 record because `laohan75` and `laohan76` are absent; token and email restored using the retained authenticated browser session) |
| Phoebee1 | Phoebee@asu.edu | 768.96 | 1000 | 231.04 | `laohan78` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Virginiaaz | Virginiaa@asu.edu | 609.17 | 1000 | 390.83 | `laohan79` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Francescazzz | Francesca@asu.edu | 800.82 | 1000 | 199.18 | `laohan80` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Okuneva1 | Okuneva@asu.edu | 558.58 | 1000 | 441.42 | `laohan81` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Stammzz | Stammz@asu.edu | 625.79 | 1000 | 374.21 | `laohan82` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Mayertzz | Mayertzz@asu.edu | 993.62 | 1000 | 6.38 | `laohan83` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored below the 10-credit scheduling threshold) |
| Goodwinzz | Goodwinz@asu.edu | 579.19 | 1000 | 420.81 | `laohan84` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Koeppzz | Koeppzz@asu.edu | 883.74 | 1000 | 116.26 | `laohan85` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
| Kuhlman1 | Kuhlman@asu.edu | 863.52 | 1000 | 136.48 | `laohan86` (operator-confirmed sequential mapping after GitHub password rotation using the retained authenticated browser session; token and email restored) |
