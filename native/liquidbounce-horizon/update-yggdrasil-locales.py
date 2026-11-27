#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
base='liquidbounce.module.yggdrasilSignatureFix.'
help={
 'en_us':'For skin texture URLs containing 127.0.0.1, force the slim arm model. Other texture URLs retain their original model. Signature and texture-domain bypasses remain active independently of this option.',
 'zh_cn':'皮肤纹理网址包含 127.0.0.1 时强制使用瘦手臂模型。其他纹理网址保留原模型。关闭此选项后，签名状态和纹理域名检查的绕过功能仍然生效。',
 'zh_tw':'皮膚紋理網址包含 127.0.0.1 時強制使用細手臂模型。其他紋理網址保留原模型。關閉此選項後，簽章狀態和紋理網域檢查的略過功能仍然生效。',
}
for language in ('zh_cn','zh_tw'):
 target=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json'
 data=json.loads(target.read_text(encoding='utf8'))
 upstream=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
 data['descriptions'][base+'description']=upstream[base+'description']
 data['descriptions'][base+'forceSlimModel.description']=help[language]
 data['labels'].setdefault('ForceSlimModel','强制瘦手臂模型' if language=='zh_cn' else '強制細手臂模型')
 if language=='zh_cn':
  english=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
  data['englishDescriptions'][base+'description']=english[base+'description']
  data['englishDescriptions'][base+'forceSlimModel.description']=help['en_us']
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
