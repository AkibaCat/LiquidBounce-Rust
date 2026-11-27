from pathlib import Path
import argparse,json
from display_locale_schema import put_module

CN={'enabled':('启用','启用这一方块标记渲染器。'),'clump':('合并相邻方块','隐藏相邻标记的内部表面和边。'),'startSize':('起始大小','淡入开始时的方块大小倍数。'),'startCurve':('起始大小缓动','淡入期间改变大小的缓动曲线。'),'endSize':('结束大小','淡出结束时的方块大小倍数。'),'endCurve':('结束大小缓动','淡出期间改变大小的缓动曲线。'),'fadeInCurve':('淡入缓动','方块出现时改变透明度的缓动曲线。'),'fadeOutCurve':('淡出缓动','方块消失时改变透明度的缓动曲线。'),'inTime':('淡入时间','出现动画的持续时间，单位为毫秒。'),'outTime':('淡出时间','消失动画的持续时间，单位为毫秒。'),'color':('填充颜色','方块表面的颜色。'),'outlineColor':('轮廓颜色','方块边缘线条的颜色。')}
TW={'enabled':('啟用','啟用這個方塊標記繪製器。'),'clump':('合併相鄰方塊','隱藏相鄰標記的內部表面和邊。'),'startSize':('起始大小','淡入開始時的方塊大小倍數。'),'startCurve':('起始大小緩動','淡入期間改變大小的緩動曲線。'),'endSize':('結束大小','淡出結束時的方塊大小倍數。'),'endCurve':('結束大小緩動','淡出期間改變大小的緩動曲線。'),'fadeInCurve':('淡入緩動','方塊出現時改變透明度的緩動曲線。'),'fadeOutCurve':('淡出緩動','方塊消失時改變透明度的緩動曲線。'),'inTime':('淡入時間','出現動畫的持續時間，單位為毫秒。'),'outTime':('淡出時間','消失動畫的持續時間，單位為毫秒。'),'color':('填滿顏色','方塊表面的顏色。'),'outlineColor':('輪廓顏色','方塊邊緣線條的顏色。')}
EN={'enabled':'Enable this block-marker renderer.',
    'clump':'Hide internal faces and edges between adjacent markers.',
    'startSize':'Block-size multiplier at the start of the fade-in animation.',
    'startCurve':'Size easing curve during fade-in.',
    'endSize':'Block-size multiplier at the end of the fade-out animation.',
    'endCurve':'Size easing curve during fade-out.',
    'fadeInCurve':'Opacity easing curve while the block marker appears.',
    'fadeOutCurve':'Opacity easing curve while the block marker disappears.',
    'inTime':'Appearance-animation duration, measured in milliseconds.',
    'outTime':'Disappearance-animation duration, measured in milliseconds.',
    'color':'Fill color for block-marker faces.',
    'outlineColor':'Line color for block-marker edges.'}

def update(language,path):
    target=json.loads(path.read_text(encoding='utf8'));data={};tw=language=='zh_tw'
    for module,group,title,description,group_title in [('prophuntESP','renderBlockUpdates','躲貓貓透視'if tw else'躲猫猫透视','標記下落方塊和伺服器方塊更新。'if tw else'标记下落方块和服务器方块更新。','方塊更新標記'if tw else'方块更新标记'),('voidEsp','render','虛空透視'if tw else'虚空透视','標記附近向下掃描後可能通往虛空的位置。'if tw else'标记附近向下扫描后可能通往虚空的位置。','方塊標記'if tw else'方块标记')]:
        prefix=f'liquidbounce.module.{module}';data[prefix]=title;data[prefix+'.description']=description;data[prefix+'.'+group]=group_title;data[prefix+'.'+group+'.description']='設定方塊標記的外觀與動畫。'if tw else'设置方块标记的外观和动画。'
        for key,(label,desc)in(TW if tw else CN).items():data[f'{prefix}.{group}.{key}']=label;data[f'{prefix}.{group}.{key}.description']=desc
    p='liquidbounce.module.prophuntESP.'
    for key,value in {'tracking':'追蹤來源'if tw else'追踪来源','tracking.description':'選擇至少一種要標記的方塊更新來源。'if tw else'选择至少一种要标记的方块更新来源。','tracking.choice.fallingBlocks':'下落方塊'if tw else'下落方块','tracking.choice.blockUpdates':'單一方塊更新'if tw else'单个方块更新','tracking.choice.chunkDeltaUpdates':'區段方塊更新'if tw else'区段方块更新'}.items():data[p+key]=value
    p='liquidbounce.module.voidEsp.'
    for key,value in {'yThreshold':'向下掃描深度'if tw else'向下扫描深度','yThreshold.description':'尋找立足點及檢查虛空時向下掃描的格數。'if tw else'寻找立足点和检查虚空时向下扫描的格数。','rangeSide':'側向範圍'if tw else'侧向范围','rangeSide.description':'相對面向方向向左右掃描的格數。'if tw else'相对面向方向向左右扫描的格数。','rangeFacing':'前後範圍'if tw else'前后范围','rangeFacing.description':'沿面向方向向前後掃描的格數。'if tw else'沿面向方向向前后扫描的格数。'}.items():data[p+key]=value
    for module,group,name in [('prophuntESP','renderBlockUpdates','ProphuntESP'),('voidEsp','render','VoidESP')]:
        prefix=f'liquidbounce.module.{module}'
        entries={key[len(prefix):].lstrip('.'):value for key,value in data.items()if key==prefix or key.startswith(prefix+'.')}
        english={f'{group}.{key}.description':text for key,text in EN.items()}
        english[f'{group}.description']='Configure the appearance and animation of block markers.'
        if module=='prophuntESP':
            english.update({'description':'Mark falling blocks and server block updates.',
                'tracking.description':'Select at least one source of block updates to mark.'})
        else:
            english.update({'description':'Mark nearby positions that may lead into the void after a downward scan.',
                'yThreshold.description':'Downward scan depth for finding support and checking for the void.',
                'rangeSide.description':'Sideways scan distance relative to the facing direction.',
                'rangeFacing.description':'Forward and backward scan distance along the facing direction.'})
        put_module(target,prefix,name,entries,english)
    path.write_text(json.dumps(target,ensure_ascii=False,indent=2)+'\n',encoding='utf8')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--directory',required=True);args=parser.parse_args()
    for language in ['zh_cn','zh_tw']:update(language,Path(args.directory)/f'display_{language}.json')
