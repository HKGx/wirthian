import {
  LRLanguage,
  LanguageSupport,
  foldNodeProp,
  foldInside,
  indentNodeProp,
  continuedIndent,
} from "@codemirror/language";
import { completeFromList, snippetCompletion } from "@codemirror/autocomplete";
import { styleTags, tags as t } from "@lezer/highlight";
import { parser } from "./wirthian.grammar";
import type { Completion } from "@codemirror/autocomplete";

const parserWithMetadata = parser.configure({
  props: [
    styleTags({
      "and or not true false if then elif else for to do break continue begin end exit print readint readstr readbool substring length position concatenate string integer boolean":
        t.keyword,
      Number: t.number,
      String: t.string,
      Identifier: t.variableName,
      ArithOp: t.operator,
      CompareOp: t.operator,
      AssignOp: t.operator,
      "( )": t.paren,
      ";": t.separator,
      ",": t.separator,
    }),
    indentNodeProp.add({
      BlockStatement: continuedIndent({ except: /^\s*({|end\b)/ }),
    }),
    foldNodeProp.add({
      BlockStatement: foldInside,
    }),
  ],
});

export const wirthianLanguage = LRLanguage.define({
  parser: parserWithMetadata,
  languageData: {
    closeBrackets: {
      brackets: ["(", ")"],
    },
    commentTokens: {},
  },
});

const constants = ["false", "true"].map((n) => ({
  label: n,
  type: "constant",
})) satisfies Completion[];

const keywords = [
  "if",
  "then",
  "elif",
  "else",
  "for",
  "to",
  "do",
  "break",
  "continue",
  "begin",
  "end",
  "exit",
].map((n) => ({
  label: n,
  type: "keyword",
})) satisfies Completion[];

const snippets = [
  snippetCompletion("for ${name} := 1 to ${end} do ", {
    label: "for",
    type: "keyword",
  }),
  snippetCompletion("begin\n\t${}\nend;", {
    label: "begin ... end",
    type: "keyword",
  }),
  ...["string", "integer", "boolean"].map((n) =>
    snippetCompletion(`${n} \${name};`, {
      label: n,
      detail: "variable",
      type: "keyword",
    }),
  ),
  ...["readint", "readstr", "readbool"].map((n) =>
    snippetCompletion(n, {
      label: n,
      detail: "input",
      type: "keyword",
    }),
  ),
  snippetCompletion("print(${expr})", {
    label: "print",
    type: "function",
  }),
  snippetCompletion("length(${string})", {
    label: "length",
    type: "function",
  }),
  snippetCompletion("position(${haystack}, ${needle})", {
    label: "position",
    type: "function",
  }),
  snippetCompletion("concatenate(${left}, ${right})", {
    label: "concatenate",
    type: "function",
  }),
  snippetCompletion("substring(${string}, ${pos}, ${len})", {
    label: "substring",
    type: "function",
  }),
] satisfies Completion[];

const completion = wirthianLanguage.data.of({
  autocomplete: completeFromList([...snippets, ...constants, ...keywords]),
});

export function wirthian() {
  return new LanguageSupport(wirthianLanguage, [completion]);
}
