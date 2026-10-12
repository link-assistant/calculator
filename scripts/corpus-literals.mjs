/** Read literal assertions without evaluating or loading upstream code. */
export function decodeLiteral(body) {
  return body.replace(/\\(?:u\{([\da-f]+)\}|u([\da-f]{4})|x([\da-f]{2})|([\s\S]))/gi,
    (match, codepoint, unicode, hex, char) => {
      if (codepoint) return String.fromCodePoint(parseInt(codepoint, 16));
      if (unicode || hex) return String.fromCharCode(parseInt(unicode || hex, 16));
      return { n: '\n', t: '\t', r: '\r', b: '\b', f: '\f', v: '\v', '\\': '\\', '"': '"', "'": "'", '`': '`', 0: '\0' }[char] ?? match;
    });
}

function quoteEnd(text, start) {
  const quote = text[start];
  for (let i = start + 1; i < text.length; i++) {
    if (text[i] === '\\') i++;
    else if (text[i] === quote) return i + 1;
  }
  throw new Error('Unterminated upstream string literal');
}

function regexEnd(text, start) {
  let inClass = false;
  for (let i = start + 1; i < text.length; i++) {
    if (text[i] === '\\') i++;
    else if (text[i] === '[') inClass = true;
    else if (text[i] === ']') inClass = false;
    else if (text[i] === '/' && !inClass) return i + 1;
  }
  throw new Error('Unterminated upstream regular expression');
}

/** Preserve line numbers, quotes and URLs while dropping commented-out tests. */
export function stripComments(text, language = 'js') {
  let output = '';
  for (let i = 0; i < text.length;) {
    if (text[i] === '"' || (language === 'js' && /['`]/.test(text[i]))) {
      const end = quoteEnd(text, i);
      output += text.slice(i, end);
      i = end;
    } else if (text.startsWith('/*', i)) {
      const end = text.indexOf('*/', i + 2);
      if (end < 0) throw new Error('Unterminated upstream comment');
      output += text.slice(i, end + 2).replace(/[^\n]/g, ' ');
      i = end + 2;
    } else if (text.startsWith(language === 'purs' ? '--' : '//', i)) {
      const end = text.indexOf('\n', i);
      const limit = end < 0 ? text.length : end;
      output += ' '.repeat(limit - i);
      i = limit;
    } else if (language === 'js' && text[i] === '/' && /(?:[=(,:!{;?]|\breturn|=>)\s*$/.test(output)) {
      // Regex assertions can contain apostrophes and apparent call syntax;
      // they are not literal calculator expectations.
      const end = regexEnd(text, i);
      output += text.slice(i, end).replace(/[^\n]/g, ' ');
      i = end;
    } else {
      output += text[i++];
    }
  }
  return output;
}

export function balancedEnd(text, start) {
  const closing = { '(': ')', '[': ']', '{': '}' };
  const stack = [];
  for (let i = start; i < text.length; i++) {
    if (/['"`]/.test(text[i])) i = quoteEnd(text, i) - 1;
    else if (closing[text[i]]) stack.push(closing[text[i]]);
    else if (/[)\]}]/.test(text[i])) {
      if (stack.pop() !== text[i]) throw new Error('Unbalanced upstream assertion');
      if (!stack.length) return i + 1;
    }
  }
  throw new Error('Unterminated upstream assertion');
}

export function splitArguments(text) {
  const args = [];
  let start = 0;
  for (let i = 0; i < text.length; i++) {
    if (/['"`]/.test(text[i])) i = quoteEnd(text, i) - 1;
    else if (/[([{]/.test(text[i])) i = balancedEnd(text, i) - 1;
    else if (text[i] === ',') {
      args.push(text.slice(start, i).trim());
      start = i + 1;
    }
  }
  const last = text.slice(start).trim();
  if (last) args.push(last);
  return args;
}

export function* calls(text, names) {
  const pattern = new RegExp(`\\b(?:${names.join('|')})\\s*\\(`, 'y');
  for (let i = 0; i < text.length; i++) {
    if (/['"`]/.test(text[i])) {
      i = quoteEnd(text, i) - 1;
      continue;
    }
    pattern.lastIndex = i;
    const match = pattern.exec(text);
    if (!match) continue;
    const start = i + match[0].lastIndexOf('(');
    const end = balancedEnd(text, start);
    yield { args: splitArguments(text.slice(start + 1, end - 1)), index: i,
      line: text.slice(0, i).split('\n').length };
    i = end - 1;
  }
}

export function stringLiteral(text) {
  text = text.trim();
  // Qt wrappers and C++ UTF-8 prefixes still represent literal text.
  const wrapper = text.match(/^(?:QString::fromUtf8|QStringLiteral|QLatin1String)\((.*)\)$/s);
  if (wrapper) return stringLiteral(wrapper[1]);
  text = text.replace(/^u8(?=")/, '');
  if (!/^['"`]/.test(text) || quoteEnd(text, 0) !== text.length) return null;
  if (text[0] === '`' && text.includes('${')) return null;
  return decodeLiteral(text.slice(1, -1));
}

export function singleLine(text) {
  return text.trim() !== '' && !/[\u0000-\u0008\u000a-\u001f\u007f]/.test(text);
}

/** Only literals have known display values; variables and API internals do not. */
export function displayLiteral(text) {
  const str = stringLiteral(text);
  if (str !== null) return str;
  if (/^(?:[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[-+]?\d+)?|true|false|NaN|Infinity|-Infinity)$/i.test(text)) return text;
  if (text.startsWith('[') && balancedEnd(text, 0) === text.length) {
    const values = splitArguments(text.slice(1, -1)).map(displayLiteral);
    if (values.every((value) => value !== null)) return `[${values.join(', ')}]`;
  }
  return null;
}
