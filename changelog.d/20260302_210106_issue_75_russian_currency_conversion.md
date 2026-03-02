## Added

- Multilingual currency conversion expressions are now supported across all 7 languages
  the calculator UI supports (English, Russian, Chinese, Hindi, Arabic, German, French).
  Fixes #75.

  - **Russian** (Русский): The preposition "в" (meaning "in/into") is recognized as a
    conversion keyword (e.g., `1000 рублей в долларах`). All Russian grammatical cases
    for major currencies are supported: рубль/рублей/рублях (RUB), доллар/долларах (USD),
    евро (EUR), фунт/фунтах (GBP), юань/юанях (CNY), иена/иенах (JPY), рупия/рупиях (INR).

  - **French** (Français): The preposition "en" (meaning "in/into") is recognized as a
    conversion keyword (e.g., `1000 dollars en euros`). French currency names are supported:
    dollar/dollars (USD), euro/euros (EUR), livre/livres (GBP), yen/yens (JPY),
    franc/francs (CHF), yuan/yuans (CNY), rouble/roubles (RUB), roupie/roupies (INR),
    plus extended forms like "dollar américain", "livre sterling", "franc suisse",
    "rouble russe", "roupie indienne".

  - **Chinese** (中文): Conversion keywords "换成" (exchange into), "兑换成", "转换为",
    "兑成", "转为" are recognized (e.g., `1000 美元 换成 欧元`). Chinese currency names
    are supported: 美元/美金 (USD), 欧元 (EUR), 英镑 (GBP), 日元/日圆 (JPY),
    瑞士法郎/法郎 (CHF), 人民币/元/块 (CNY), 卢布 (RUB), 卢比/印度卢比 (INR).

  - **Hindi** (हिन्दी): The postposition "में" (meaning "in") is recognized as a
    conversion keyword (e.g., `1000 डॉलर में यूरो`). Hindi currency names are supported:
    डॉलर (USD), यूरो (EUR), पाउंड (GBP), येन (JPY), फ्रैंक (CHF), युआन (CNY),
    रूबल (RUB), रुपया/रुपये/रुपयों (INR).

  - **Arabic** (العربية): The preposition "إلى" (meaning "to/into") is recognized as a
    conversion keyword (e.g., `1000 دولار إلى يورو`). Arabic currency names are supported:
    دولار/دولارات (USD), يورو (EUR), جنيه/جنيهات (GBP), ين (JPY),
    فرنك (CHF), يوان (CNY), روبل (RUB), روبية/روبيات (INR).

  - **German** (Deutsch): Uses the same "in" preposition as English (no change needed).
    German currency names are supported: Dollar/Dollars (USD), Euro/Euros (EUR),
    Pfund/Pfund Sterling (GBP), Yen (JPY), Franken/Schweizer Franken (CHF),
    Yuan (CNY), Rubel/Rubeln (RUB), Rupie/Rupien (INR).
