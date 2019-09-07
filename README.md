# regex-vm

VM型正規表現エンジン

## 機能

- 連接
- 選択
- 繰り返し
    - 任意長
    - 1文字以上
    - 範囲で指定した回数
- 省略
- 文字クラス
- グループ化
- 任意文字マッチ

## 例(実行結果)
```
Pattern: ^a?b+[cde]{1,3}|hoge(.+)$
--------------------------------
AST: Ok(PrefixSuffix(true, Branch([Connect([Maybe(Literal('a')), RepeatPlus(Literal('b')), RepeatRange(CharClass(false, ['c', 'd', 'e']), 1, 3)]), Connect([Literal('h'), Literal('o'), Literal('g'), Literal('e'), Group(Branch([Connect([RepeatPlus(AnyChar)])]))])]), true))
--------------------------------
00: MatchPos(Front)
01: Branch(2, 12)
02: Branch(3, 4)
03: MatchChar(Literal('a'))
04: MatchChar(Literal('b'))
05: Branch(4, 6)
06: MatchChar(CharClass(false, ['c', 'd', 'e']))
07: Branch(8, 9)
08: MatchChar(CharClass(false, ['c', 'd', 'e']))
09: Branch(10, 11)
10: MatchChar(CharClass(false, ['c', 'd', 'e']))
11: Jump(20)
12: MatchChar(Literal('h'))
13: MatchChar(Literal('o'))
14: MatchChar(Literal('g'))
15: MatchChar(Literal('e'))
16: GroupParenL
17: MatchChar(Any)
18: Branch(17, 19)
19: GroupParenR
20: MatchPos(Back)
21: Finish
--------------------------------
Text: abc
Result: Some([(0, 3)])
--------------------------------
Text: bbcc
Result: Some([(0, 4)])
--------------------------------
Text: ac
Result: None
--------------------------------
Text: bcde
Result: Some([(0, 4)])
--------------------------------
Text: xyz
Result: None
--------------------------------
Text: hogeXXXX
Result: Some([(0, 8), (4, 8)])
```