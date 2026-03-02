## Added

- Russian-language currency conversion expressions are now supported. The expression
  `1000 рублей в долларах` is now correctly recognized as a conversion from RUB to USD.
  - The Russian preposition "в" (meaning "in/into") is now recognized as a conversion
    keyword, equivalent to the English "in"/"as"/"to" keywords.
  - All Russian grammatical cases for major currencies are now supported:
    - USD: доллар, доллара, долларе, доллары, долларов, долларам, доллару, долларом,
      долларами, долларах
    - EUR: евро (indeclinable)
    - GBP: фунт, фунта, фунте, фунты, фунтов, фунтам, фунту, фунтом, фунтами, фунтах
    - CNY: юань, юаня, юане, юани, юаней, юаням, юаню, юанем, юанями, юанях
    - JPY: иена, иены, иене, иену, иеной, иенами, иенах, иен
    - RUB: added missing cases рубле, рублях, рублям, рублями (previously only
      рубль, рубля, рублей, рублю, рублём, рублем, рубли were supported)
  Fixes #75.
