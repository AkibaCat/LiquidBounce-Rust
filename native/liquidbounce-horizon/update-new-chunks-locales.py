from pathlib import Path
import argparse, json
from display_locale_schema import put_module

CN = {'': '新区块', 'description': '根据流动液体更新标记可能新生成的区块。', 'renderDistance': '渲染距离', 'renderDistance.description': '显示区块标记的最大水平距离，单位为区块。', 'renderY': '渲染高度', 'renderY.description': '区块标记平面的世界高度。', 'autoY': '自动高度', 'autoY.description': '在玩家下方 100 格显示标记平面。', 'smooth': '平滑颜色', 'smooth.description': '根据附近新区块的分布混合旧区块颜色。', 'persist': '保留记录', 'persist.description': '区块卸载后继续保留识别结果。', 'newColor': '新区块颜色', 'newColor.description': '被流动液体更新标记为新区块时的颜色。', 'oldColor': '旧区块颜色', 'oldColor.description': '已加载且尚未识别为新区块时的颜色。'}
TW = {'': '新區塊', 'description': '根據流動液體更新標記可能新生成的區塊。', 'renderDistance': '繪製距離', 'renderDistance.description': '顯示區塊標記的最大水平距離，單位為區塊。', 'renderY': '繪製高度', 'renderY.description': '區塊標記平面的世界高度。', 'autoY': '自動高度', 'autoY.description': '在玩家下方 100 格顯示標記平面。', 'smooth': '平滑顏色', 'smooth.description': '根據附近新區塊的分布混合舊區塊顏色。', 'persist': '保留記錄', 'persist.description': '區塊卸載後繼續保留識別結果。', 'newColor': '新區塊顏色', 'newColor.description': '被流動液體更新標記為新區塊時的顏色。', 'oldColor': '舊區塊顏色', 'oldColor.description': '已載入且尚未識別為新區塊時的顏色。'}

EN = {'description':'Mark potentially new chunks from flowing-fluid updates.',
      'renderDistance.description':'Maximum horizontal distance for chunk markers, measured in chunks.',
      'renderY.description':'World height of the chunk-marker plane.',
      'autoY.description':'Render the marker plane 100 blocks below the player.',
      'smooth.description':'Blend old-chunk colors according to the distribution of nearby new chunks.',
      'persist.description':'Retain detected chunk classifications after chunks unload.',
      'newColor.description':'Marker color for chunks classified as new by flowing-fluid updates.',
      'oldColor.description':'Marker color for loaded chunks that have not been classified as new.'}

def update(language, path):
    data = json.loads(path.read_text(encoding='utf8'))
    put_module(data, 'liquidbounce.module.newChunks', 'NewChunks', CN if language == 'zh_cn' else TW, EN)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--directory', required=True)
    args = parser.parse_args()
    for language in ['zh_cn', 'zh_tw']:
        update(language, Path(args.directory) / f'display_{language}.json')
