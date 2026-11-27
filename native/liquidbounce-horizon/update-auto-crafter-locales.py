"""AutoCrafter Simplified/Traditional Chinese live captions and help."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'description':('Automatically craft ordered items using the open recipe book of an allowed container.','使用允许容器中已打开的配方书，按列表顺序自动合成物品。','使用允許容器中已開啟的配方書，依清單順序自動合成物品。'),
 'itemsToCraft':('Ordered crafting targets; recipes consuming a later target are skipped to avoid loops.','按顺序合成的目标；跳过消耗列表后续目标的配方以避免循环。','依序合成的目標；略過消耗清單後續目標的配方以避免循環。'),
 'delay':('Ticks to wait after placing, collecting or throwing a recipe result.','放置配方、收取或丢弃产物后的等待刻数。','放置配方、收取或丟棄產物後的等待刻數。'),
 'stackCrafting':('Place the maximum available recipe amount.','一次放入可合成的最大数量。','一次放入可合成的最大數量。'),
 'sequentialCrafting':('End the current pass after one recipe action; furnaces always do this.','每轮只处理一个配方操作；熔炉始终按此方式处理。','每輪僅處理一個配方操作；熔爐一律以此方式處理。'),
 'onFull':('When inventory capacity cannot hold the result, disable, close, wait or throw it.','背包空间不足以收取产物时，选择关闭模块、关闭容器、等待或丢弃。','背包空間不足以收取產物時，選擇停用模組、關閉容器、等待或丟棄。'),
 'allowedContainers':('Allow inventory, crafting table, furnace, blast furnace or smoker recipe books.','允许背包、工作台、熔炉、高炉或烟熏炉的配方书。','允許背包、工作台、熔爐、高爐或煙燻爐的配方書。'),
}
LABELS={'AutoCrafter':('自动合成','自動合成'),'ItemsToCraft':('合成目标列表','合成目標清單'),'StackCrafting':('批量合成','批次合成'),'SequentialCrafting':('逐次合成','逐次合成'),'OnFull':('背包满时','背包滿時'),'AllowedContainers':('允许的容器','允許的容器'),'CraftingTable':('工作台','工作臺'),'BlastFurnace':('高炉','高爐'),'Smoker':('烟熏炉','煙燻爐'),'CloseScreen':('关闭容器','關閉容器')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  p=folder/f'display_{locale}.json';d=json.loads(p.read_text(encoding='utf8'));suffix=lambda k:k if k=='description'else k+'.description'
  put_module(d,'liquidbounce.module.autoCrafter','AutoCrafter',{'':LABELS['AutoCrafter'][index-1],**{suffix(k):v[index]for k,v in TEXTS.items()}},{suffix(k):v[0]for k,v in TEXTS.items()});d['labels'].update({k:v[index-1]for k,v in LABELS.items()});p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().directory)
