"""StrongholdFinder settings/help in the current game language."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'description':('Track Eye of Ender throws and estimate stronghold chunks using a Bayesian posterior.','追踪末影之眼的投掷轨迹，使用贝叶斯后验概率推算要塞区块。','追蹤終界之眼的投擲軌跡，以貝氏後驗機率推算要塞區塊。'),
 'sigma':('Angular measurement uncertainty in degrees.','角度测量的不确定性，单位为度。','角度測量的不確定性，單位為度。'),
 'hypothesisCount':('Number of deterministic stronghold layouts evaluated for each posterior.','每次预测所计算的确定性要塞布局数量。','每次預測所計算的確定性要塞布局數量。'),
 'requireSameStrongholdAcrossThrows':('Require every throw in one hypothesis to target the same stronghold.','要求同一假设中的每次投掷都指向同一个要塞。','要求同一假設中的每次投擲都指向同一個要塞。'),
 'sampleDelayTicks':('Wait this many player ticks before sampling a spawned eye.','末影之眼生成后，等待指定玩家刻再采样。','終界之眼生成後，等待指定玩家刻再採樣。'),
 'minEyeHorizontalSpeed':('Wait until the eye reaches this minimum horizontal speed.','等待末影之眼达到此最小水平速度。','等待終界之眼達到此最小水平速度。'),
 'maxSampleAgeTicks':('Maximum age of a pending throw when matching its spawned eye.','匹配生成的末影之眼时，待处理投掷记录允许的最大刻数。','配對生成的終界之眼時，待處理投擲紀錄允許的最大刻數。'),
 'maxEyeSpawnDistance':('Maximum horizontal distance between a throw and its spawned eye.','投掷位置与末影之眼生成位置之间的最大水平距离。','投擲位置與終界之眼生成位置之間的最大水平距離。'),
 'showTopCandidates':('Number of highest probability candidate chunks to retain and render.','保留并显示概率最高的候选区块数量。','保留並顯示機率最高的候選區塊數量。'),
 'renderRays':('Draw a 2048 block direction ray for each recorded measurement.','为每个测量样本绘制长 2048 格的方向射线。','為每個測量樣本繪製長 2048 格的方向射線。'),
 'renderBestChunk':('Highlight the most likely stronghold chunk.','高亮最可能包含要塞的区块。','醒目顯示最可能包含要塞的區塊。'),
 'renderTopChunks':('Highlight additional high probability candidate chunks.','高亮其他高概率候选区块。','醒目顯示其他高機率候選區塊。'),
 'announcePrediction':('Notify when the best predicted chunk changes.','最优预测区块变化时显示通知。','最佳預測區塊變更時顯示通知。'),
 'resetOnWorldChange':('Clear samples, predictions and portal highlights when the world changes.','切换世界时清除样本、预测与传送门高亮。','切換世界時清除樣本、預測與傳送門標示。'),
}
LABELS={'StrongholdFinder':('要塞定位','要塞定位'),'Sigma':('角度误差','角度誤差'),'HypothesisCount':('布局假设数量','布局假設數量'),'RequireSameStrongholdAcrossThrows':('要求投掷指向同一要塞','要求投擲指向同一要塞'),'SampleDelayTicks':('采样延迟刻数','採樣延遲刻數'),'MinEyeHorizontalSpeed':('最小水平速度','最小水平速度'),'MaxSampleAgeTicks':('投掷记录最大刻数','投擲紀錄最大刻數'),'MaxEyeSpawnDistance':('最大生成距离','最大生成距離'),'ShowTopCandidates':('候选区块数量','候選區塊數量'),'RenderRays':('显示测量射线','顯示測量射線'),'RenderBestChunk':('显示最优区块','顯示最佳區塊'),'RenderTopChunks':('显示其他候选区块','顯示其他候選區塊'),'AnnouncePrediction':('通知预测结果','通知預測結果'),'ResetOnWorldChange':('切换世界时重置','切換世界時重設')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  texts={('description' if k=='description' else k+'.description'):v[index] for k,v in TEXTS.items()}
  english={('description' if k=='description' else k+'.description'):v[0] for k,v in TEXTS.items()}
  put_module(data,'liquidbounce.module.strongholdFinder','StrongholdFinder',{'':'要塞定位',**texts},english)
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()});path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
