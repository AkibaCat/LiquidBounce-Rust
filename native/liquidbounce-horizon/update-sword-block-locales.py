"""SwordBlock English and live Simplified/Traditional setting descriptions."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'description':('Show a shield block as main-hand sword blocking.','将持盾格挡显示为主手剑格挡。','將持盾格擋顯示為主手劍格擋。'),
 'onlyVisual.description':('Change the displayed pose only. Native 26.3 already supports the blocks-attacks component.','仅修改显示姿态。原生 26.3 已支持攻击格挡组件。','僅修改顯示姿態。原生 26.3 已支援攻擊格擋元件。'),
 'fakeOnPressing.description':('Show the sword block pose while the use key is physically held.','按住使用键时显示剑格挡姿态。','按住使用鍵時顯示劍格擋姿態。'),
 'hideShieldSlot.description':('Also hide the offhand hotbar slot when SwordBlock hides the shield.','剑格挡隐藏盾牌时，同时隐藏快捷栏的副手槽位。','劍格擋隱藏盾牌時，同時隱藏快捷列的副手欄位。'),
 'applyToThirdPersonView.description':('Apply sword blocking and shield hiding to the local player in third person. This also gates the AutoBlock pose.','在第三人称中显示本地玩家的剑格挡并隐藏盾牌，也控制自动格挡姿态。','在第三人稱中顯示本機玩家的劍格擋並隱藏盾牌，也控制自動格擋姿態。'),
 'alwaysHideShield.description':('Hide an offhand shield even when the main hand is not holding a sword.','主手未持剑时也隐藏副手盾牌。','主手未持劍時也隱藏副手盾牌。'),
}
LABELS={'SwordBlock':('剑格挡','劍格擋'),'OnlyVisual':('仅显示效果','僅顯示效果'),'FakeOnPressing':('按键时模拟格挡','按鍵時模擬格擋'),'HideShieldSlot':('隐藏盾牌槽位','隱藏盾牌欄位'),'ApplyToThirdPersonView':('应用至第三人称','套用至第三人稱'),'AlwaysHideShield':('始终隐藏盾牌','一律隱藏盾牌')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  put_module(data,'liquidbounce.module.swordBlock','SwordBlock',{'':LABELS['SwordBlock'][index-1],**{k:v[index]for k,v in TEXTS.items()}},{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()});path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
