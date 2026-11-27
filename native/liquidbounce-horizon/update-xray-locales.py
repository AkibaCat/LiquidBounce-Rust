from pathlib import Path
import json
HERE=Path(__file__).resolve().parent
for code,index in [('zh_cn',1),('zh_tw',2)]:
 p=HERE.parent/'liquidbounce-interop/resources/localization'/f'display_{code}.json';d=json.loads(p.read_text(encoding='utf8'))
 d['moduleNames']['XRay']=['矿物透视','礦物透視'][index-1]
 for k,values in {'ExposedOnly':('仅显示暴露方块','僅顯示暴露方塊'),'BackgroundOpacity':('背景不透明度','背景不透明度')}.items():d['labels'][k]=values[index-1]
 for key,values in {
 'fullBright.description':('Disable terrain ambient occlusion and light the world while XRay is active.','透视启用时关闭地形环境光遮蔽并提高世界亮度。','透視啟用時關閉地形環境光遮蔽並提高世界亮度。'),
 'exposedOnly.description':('Only show listed blocks beside a block that does not conduct redstone.','仅显示六个相邻方块中至少一个不传导红石的列表方块。','僅顯示六個相鄰方塊中至少一個不傳導紅石的清單方塊。'),
 'backgroundOpacity.description':('Opacity of other non-air blocks, from 0 (hidden) to 255 (opaque).','其他非空气方块的不透明度；0 为隐藏，255 为完全不透明。','其他非空氣方塊的不透明度；0 為隱藏，255 為完全不透明。'),
 'blocks.description':('Blocks to reveal. Changes rebuild loaded terrain. Manage the list with the xray command.','需要显示的方块；修改后重建已加载地形。也可使用 xray 命令管理列表。','需要顯示的方塊；修改後重建已載入地形。也可使用 xray 指令管理清單。')}.items():
  key='liquidbounce.module.xRay.'+key;d['descriptions'][key]=values[index]
  if index==1:d['englishDescriptions'][key]=values[0]
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(code,'XRay settings and descriptions')
