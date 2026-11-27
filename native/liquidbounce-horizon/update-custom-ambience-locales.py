"""Complete source settings and inactive modes, Simplified/Traditional Chinese."""
from pathlib import Path
import argparse,json,sys
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'display_locale_schema.py').is_file()else HERE.parents[1]
sys.path.insert(0,str(NATIVE));from display_locale_schema import put_module
TEXTS={}
def add(path,en,cn,tw):TEXTS[(path+'.'if path else'')+'description']=(en,cn,tw)
add('','Override weather, the overworld clock, fog, lighting, sky and world particles.','自定义天气、主世界时钟、雾、光照、天空及世界粒子。','自訂天氣、主世界時鐘、霧、光照、天空及世界粒子。')
add('weather','Keep weather or select sunny, rainy, snowy or thunder. Snowy preserves the original precipitation-render suppression.','保留天气，或选择晴天、雨天、雪天及雷雨；雪天保留原始降水渲染入口的抑制行为。','保留天氣，或選擇晴天、雨天、雪天及雷雨；雪天保留原始降水繪製入口的抑制行為。')
add('time','Keep the clock or select dawn, day, noon, dusk, night or midnight. Only the overworld clock is replaced.','保留时钟，或选择黎明、白天、正午、黄昏、夜晚及午夜；只替换主世界时钟。','保留時鐘，或選擇黎明、白天、正午、黃昏、夜晚及午夜；只替換主世界時鐘。')
add('modifyPrecipitation','Modify nonzero weather-render intensity.','修改非零的降水渲染强度。','修改非零的降水繪製強度。')
add('modifyPrecipitation.enabled','Enable precipitation intensity replacement.','启用降水强度替换。','啟用降水強度替換。')
add('modifyPrecipitation.gradient','Intensity substituted whenever the original precipitation intensity is nonzero.','原始降水强度非零时使用的强度。','原始降水強度非零時使用的強度。')
add('fog','Override the actual environmental, distance, sky and cloud fog fields.','覆盖环境雾、距离雾、天空雾及云雾的实际字段。','覆寫環境霧、距離霧、天空霧及雲霧的實際欄位。')
add('fog.enabled','Enable fog ranges, optional color and background replacement.','启用雾范围、可选雾色及背景色替换。','啟用霧範圍、可選霧色及背景色替換。')
add('fog.disableWorldFog','Bind the no-fog world path while retaining custom clear and fog colors.','关闭世界雾，同时保留自定义清屏色及雾色。','關閉世界霧，同時保留自訂清除色及霧色。')
add('fog.fogColorOverride','Replace computed fog color before environmental color processing.','替换环境颜色处理前计算得到的雾色。','替換環境顏色處理前計算得到的霧色。')
add('fog.fogColorOverride.enabled','Enable the independent fog-color replacement.','启用独立的雾色替换。','啟用獨立的霧色替換。')
add('fog.fogColorOverride.color','Fog RGBA, including its original alpha.','雾的 RGBA 颜色，保留原始透明度。','霧的 RGBA 顏色，保留原始透明度。')
add('fog.backgroundColor','World framebuffer clear color; alpha zero keeps the original clear color.','世界帧缓冲清屏色；透明度为零时使用原清屏色。','世界影格緩衝清除色；透明度為零時使用原清除色。')
add('fog.environmental','Environmental fog start and end, from -16 to 2048 blocks.','环境雾的起止距离，范围为 -16～2048 格。','環境霧的起迄距離，範圍為 -16～2048 格。')
add('fog.renderDistance','Render-distance fog start and end, from 0 to 1024 blocks.','渲染距离雾的起止距离，范围为 0～1024 格。','繪製距離霧的起迄距離，範圍為 0～1024 格。')
add('fog.skyEnd','Sky fog end distance, from 0 to 1024 blocks.','天空雾的结束距离，范围为 0～1024 格。','天空霧的結束距離，範圍為 0～1024 格。')
add('fog.cloudEnd','Cloud fog end distance, from 0 to 4096 blocks.','云雾的结束距离，范围为 0～4096 格。','雲霧的結束距離，範圍為 0～4096 格。')
add('customLightmap','Edit the actual 16 by 16 lightmap texture or its shader inputs.','修改实际的 16×16 光照贴图或其着色器输入。','修改實際的 16×16 光照貼圖或其著色器輸入。')
add('customLightmap.enabled','Enable custom lightmap editing.','启用自定义光照贴图。','啟用自訂光照貼圖。')
add('customLightmap.mode','SingleColor clears the texture; Custom edits nontransparent color inputs.','单色模式清空并填充贴图；自定义模式修改不透明的颜色输入。','單色模式清除並填滿貼圖；自訂模式修改非透明的顏色輸入。')
add('customLightmap.mode.singleColor','Fill every lightmap pixel and skip the usual lightmap calculation.','填充光照贴图的全部像素，并跳过通常的光照计算。','填滿光照貼圖的全部像素，並略過一般的光照計算。')
add('customLightmap.mode.singleColor.color','RGBA written to every lightmap texel.','写入每个光照贴图像素的 RGBA 颜色。','寫入每個光照貼圖像素的 RGBA 顏色。')
add('customLightmap.mode.custom','Replace individual RGB inputs; fully transparent settings leave their input unchanged.','分别替换 RGB 输入；完全透明的设置保留其原输入。','分別替換 RGB 輸入；完全透明的設定保留其原輸入。')
for path,en,cn,tw in [('blockLightTint','Block light tint','方块光源色调','方塊光源色調'),('skyLightColor','Sky light color','天空光照颜色','天空光照顏色'),('ambientColor','Ambient light color','环境光颜色','環境光顏色'),('nightVisionColor','Night vision color','夜视颜色','夜視顏色')]:add('customLightmap.mode.custom.'+path,en+'; alpha zero keeps the original input.','替换'+cn+'；透明度为零时保留原输入。','替換'+tw+'；透明度為零時保留原輸入。')
add('skyColor','Override only the sky render state color, independently from fog color.','独立覆盖天空渲染状态的颜色，不改变雾的颜色输入。','獨立覆寫天空繪製狀態的顏色，不改變霧的顏色輸入。')
add('skyColor.enabled','Enable the sky-color replacement.','启用天空颜色替换。','啟用天空顏色替換。')
add('skyColor.color','Sky RGB; the source sky state ignores alpha.','天空 RGB 颜色；原天空状态忽略透明度。','天空 RGB 顏色；原天空狀態忽略透明度。')
add('worldParticles','Spawn original builtin particle textures around the player.','在玩家周围生成原始内置纹理粒子。','在玩家周圍產生原始內建材質粒子。')
add('worldParticles.enabled','Enable world particles; disabling clears their coordinates.','启用世界粒子；关闭时清除粒子坐标。','啟用世界粒子；關閉時清除粒子座標。')
add('worldParticles.mode','World particle mode; the pinned source contains Simple only.','世界粒子模式；固定源码只提供简单模式。','世界粒子模式；固定原始碼只提供簡單模式。')
add('worldParticles.mode.simple','Camera-facing original textured squares with lifetime, motion and wander.','使用朝向相机的原始纹理方片，支持寿命、移动与漫游。','使用朝向相機的原始材質方片，支援壽命、移動與漫遊。')
P='worldParticles.mode.simple.'
for path,en,cn,tw in [
('radius','Uniform-volume spherical-shell spawn radius.','按球壳体积均匀采样的生成半径。','依球殼體積均勻取樣的產生半徑。'),
('count','Maximum live particle count; at most one particle is created per render frame.','存活粒子上限；每个渲染帧最多创建一个粒子。','存活粒子上限；每個繪製影格最多建立一個粒子。'),
('size','Square size multiplied by eased progress.','方片尺寸乘以缓动进度。','方片尺寸乘以緩動進度。'),
('life','Animation duration and total lifetime in ticks; changing this range clears particles.','以游戏刻设置动画时长及总寿命；修改此范围会清除粒子。','以遊戲刻設定動畫時長及總壽命；修改此範圍會清除粒子。'),
('spawnTime','Minimum wall-clock milliseconds between successful spawns.','两次成功生成之间的最短实际毫秒数。','兩次成功產生之間的最短實際毫秒數。'),
('animCurve','Easing of age divided by animation duration, then clamped to zero through one.','对年龄与动画时长的比值进行缓动，并限制在零至一。','對年齡與動畫時長的比值進行緩動，並限制在零至一。'),
('canBeCovered','Use actual scene depth and depth writing for world occlusion.','使用实际场景深度及深度写入处理世界遮挡。','使用實際場景深度及深度寫入處理世界遮擋。'),
('yMotion','Optional vertical motion based on progress or age.','可按进度或年龄启用垂直移动。','可依進度或年齡啟用垂直移動。'),
('yMotion.enabled','Enable vertical particle translation.','启用粒子的垂直位移。','啟用粒子的垂直位移。'),
('yMotion.motion','Vertical distance factor, including negative downward motion.','垂直距离系数；负值向下移动。','垂直距離係數；負值向下移動。'),
('yMotion.animBy','Use eased progress or raw particle age for vertical motion.','使用缓动进度或原始粒子年龄控制垂直移动。','使用緩動進度或原始粒子年齡控制垂直移動。'),
('wander','Coordinate-hash-phased sine and cosine wander.','使用坐标哈希作为相位的正弦及余弦漫游。','使用座標雜湊作為相位的正弦及餘弦漫遊。'),
('wander.enabled','Enable original three-axis wander.','启用原始三轴漫游。','啟用原始三軸漫遊。'),
('wander.distance','Horizontal wander radius; vertical wander uses half the distance.','水平漫游半径；垂直漫游使用一半距离。','水平漫遊半徑；垂直漫遊使用一半距離。'),
('wander.speed','Angular speed multiplier for age-based wander.','按年龄计算的漫游角速度倍率。','依年齡計算的漫遊角速度倍率。'),
('particle','Choose one of the eleven original builtin PNG textures.','选择十一种原始内置 PNG 纹理之一。','選擇十一種原始內建 PNG 材質之一。'),
('color','World particle color settings.','世界粒子颜色设置。','世界粒子顏色設定。'),
('color.color','Original RGBA particle color; progress changes size rather than fading alpha.','粒子的原始 RGBA 颜色；进度改变尺寸，不渐隐透明度。','粒子的原始 RGBA 顏色；進度改變尺寸，不淡出透明度。')]:add(P+path,en,cn,tw)
LABELS={'CustomAmbience':('自定义环境','自訂環境'),'Weather':('天气','天氣'),'Time':('时间','時間'),'NoChange':('保持原样','維持原樣'),'Sunny':('晴天','晴天'),'Rainy':('雨天','雨天'),'Snowy':('雪天','雪天'),'Thunder':('雷雨','雷雨'),'Dawn':('黎明','黎明'),'Day':('白天','白天'),'Noon':('正午','正午'),'Dusk':('黄昏','黃昏'),'Night':('夜晚','夜晚'),'MidNight':('午夜','午夜'),'ModifyPrecipitation':('修改降水','修改降水'),'Gradient':('降水强度','降水強度'),'Fog':('雾','霧'),'DisableWorldFog':('关闭世界雾','關閉世界霧'),'FogColorOverride':('覆盖雾色','覆寫霧色'),'BackgroundColor':('背景颜色','背景顏色'),'Environmental':('环境雾距离','環境霧距離'),'RenderDistance':('距离雾范围','距離霧範圍'),'SkyEnd':('天空雾结束距离','天空霧結束距離'),'CloudEnd':('云雾结束距离','雲霧結束距離'),'CustomLightmap':('自定义光照贴图','自訂光照貼圖'),'SingleColor':('单色','單色'),'Custom':('自定义','自訂'),'BlockLightTint':('方块光源色调','方塊光源色調'),'SkyLightColor':('天空光照颜色','天空光照顏色'),'AmbientColor':('环境光颜色','環境光顏色'),'NightVisionColor':('夜视颜色','夜視顏色'),'SkyColor':('天空颜色','天空顏色'),'WorldParticles':('世界粒子','世界粒子'),'Life':('动画与寿命','動畫與壽命'),'SpawnTime':('生成间隔','產生間隔'),'AnimCurve':('动画曲线','動畫曲線'),'CanBeCovered':('允许遮挡','允許遮擋'),'YMotion':('垂直移动','垂直移動'),'AnimBy':('动画依据','動畫依據'),'Wander':('漫游','漫遊'),'Progress':('进度','進度'),'Age':('年龄','年齡')}
def update(lang,path):
 i=1 if lang=='zh_cn'else 2;data=json.loads(path.read_text(encoding='utf8'));put_module(data,'liquidbounce.module.customAmbience','CustomAmbience',{'':LABELS['CustomAmbience'][i-1],**{p:v[i]for p,v in TEXTS.items()}},{p:v[0]for p,v in TEXTS.items()});data['labels'].update({p:v[i-1]for p,v in LABELS.items()});path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,default=NATIVE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args()
 for lang in ['zh_cn','zh_tw']:update(lang,a.directory/f'display_{lang}.json')
