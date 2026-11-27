#!/usr/bin/env python3
"""TextFieldProtect descriptions and setting help for actual game locale switching."""
from pathlib import Path
import json
ROOT = Path(__file__).resolve().parents[2]
BASE = 'liquidbounce.module.textFieldProtect.'
HELP = {
    'en_us': 'Java regular expressions matched against each visible substring before text formatting. Defaults: ^/register.*, ^/login.*, ^/email.*. A match renders asterisks using the original module\'s display-position count. Input, clipboard and submitted text stay intact.',
    'zh_cn': '在文字格式化前，对每段可见文字执行 Java 正则表达式完整匹配。默认规则为 ^/register.*、^/login.*、^/email.*。匹配后按原模块的滚动位置数显示星号；实际输入、剪贴板和提交内容保持原样。',
    'zh_tw': '在文字格式化前，對每段可見文字執行 Java 正規表示式完整比對。預設規則為 ^/register.*、^/login.*、^/email.*。相符後依原模組的捲動位置數顯示星號；實際輸入、剪貼簿及送出內容維持原樣。',
}


def main():
    for language in ['zh_cn', 'zh_tw']:
        target = ROOT / f'native/liquidbounce-interop/resources/localization/display_{language}.json'
        data = json.loads(target.read_text(encoding='utf8'))
        source = json.loads((ROOT / f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
        data['descriptions'][BASE + 'description'] = source[BASE + 'description']
        data['descriptions'][BASE + 'patterns.description'] = HELP[language]
        data['labels']['Patterns'] = '匹配规则' if language == 'zh_cn' else '比對規則'
        if language == 'zh_cn':
            source = json.loads((ROOT / 'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
            data['englishDescriptions'][BASE + 'description'] = source[BASE + 'description']
            data['englishDescriptions'][BASE + 'patterns.description'] = HELP['en_us']
        target.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')


if __name__ == '__main__':
    main()
