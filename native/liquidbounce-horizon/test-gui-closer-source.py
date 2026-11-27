#!/usr/bin/env python3
"""Unchanged GUICloser Kotlin handler against the actual JDK25 regex engine."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def vectors():
    cases=[
        ('^Vote$',['Vote','vote','Vote\n','Vote Shop']),('Vote',['Vote','A Vote Shop']),
        ('',['','text']),('.*',['','Vote\nShop']),('(?s).*',['','Vote\nShop']),
        ('(?i)vote',['VOTE','vOtE']),('(?iu)i',['İ','ı','I']),
        (r'(Vote)\1',['VoteVote','Vote']), (r'(?<item>Vote)\k<item>',['VoteVote','Vote']),
        (r'(?<=Vote )Shop',['Vote Shop','Shop']), (r'(?<!Vote )Shop',['Vote Shop','Shop']),
        (r'Vote(?= Shop)',['Vote Shop','Vote']), (r'Vote(?! Shop)',['Vote Shop','Vote']),
        (r'(?<=a{1,3})b',['aaab','b']), (r'(?>a*)a',['aaa','a']),
        (r'a*+a',['aaa','a']), (r'a*?a',['aaa','a']), (r'(?:Vote|Shop)+',['VoteShop','Buy']),
        (r'[a-z&&[^aeiou]]+',['bcdf','abc']), (r'[a-z&&[def]&&[ef]]',['e','d']),
        (r'\QVote (1)\E',['Vote (1)','Vote 1']), (r'\Qone\E\\E\Qtwo\E',[r'one\Etwo','oneEtwo']),
        (r'\d+',['123','١٢٣']), (r'(?U)\d+',['123','١٢٣']),
        (r'\w+',['Vote_1','中文']), (r'(?U)\w+',['Vote_1','中文']),
        (r'\p{L}+',['Vote','中文']), (r'\p{javaLowerCase}+',['é','É']),
        (r'\p{IsGreek}+',['αβ','ab']), (r'\p{sc=Greek}+',['αβ','ab']),
        (r'\p{InBasic_Latin}+',['Vote','中文']), (r'\P{ASCII}+',['中文','Vote']),
        (r'\p{IsAlphabetic}+',['abc','αβ']), (r'\p{javaWhitespace}',['\u2003','\u00a0']),
        (r'\p{javaJavaIdentifierStart}',['$','1']), (r'\p{javaDefined}',['中','\u0378']),
        (r'\R',['\r\n','\u0085','\u2028','x']), (r'\h',['\t','\u2007','\n']),
        (r'\v',['\r','\u0085','x']), (r'\X',['a\u0301','😀','ab']),
        (r'\b{g}',['a\u0301','😀','']), (r'\N{GRINNING FACE}',['😀','Vote']),
        (r'\x{1f600}',['😀','Vote']), (r'\uD83D\uDE00',['😀','Vote']),
        (r'.{1}',['😀','ab']), (r'\bVote\b',['Vote','AVoteB']),
        (r'(?m)^Vote$',['Shop\nVote\nBuy','Vote\n']), (r'(?d)^Vote$',['Vote\r','Vote\n']),
        (r'Vote\Z',['Vote\n','Vote\r\n','VoteX']), (r'Vote\z',['Vote\n','Vote']),
        (r'\A(?:Vote)\z',['Vote','\nVote']), (r'\GVote',['Vote','AVote']),
        ('(?x) V o t e # comment',['Vote','vote']), ('(?i:vote)(?-i:Shop)',['VOTEShop','VOTESHOP']),
        ('a{0,2}',['','aa','aaa']), ('(a|ab)b',['abb','ab']),
        ('(?c)é',['é','e\u0301']), ('[é]',['é','e\u0301']),
        ('(?U)(?-U)\\w',['é','a']), ('(?iu)ſ',['S','s']),
        (r'(?<=a*)b',['aaab','b','acb']), (r'(?<=a+)b',['aaab','b']),
        (r'(?<=ab*)c',['abbc','ac','c']), (r'(?<!a*)b',['aaab','b','cb']),
        (r'(?iu)ß',['ß','s','S','ẞ']), (r'(?iu)İ',['İ','ı','i','I']),
        (r'(?iU)ſ',['ſ','s','S']), (r'(?iU)[i]',['İ','ı','i','I']),
        (r'(?iu)[ſ-ſ]',['ſ','s','S']), (r'(?iu)[ß-ß]',['ß','s','ẞ']),
        (r'\uD800',['\0','😀','\U000f0000']), (r'\uDC00',['\0','😀']),
        (r'\x{D800}',['\0','😀']), (r'\x{D83D}\x{DE00}',['\0','😀']),
        (r'[a-\uD800]',['a','中','😀']), (r'[^\uD800]',['\0','😀']),
        (r'\uD800?',['','\0']), (r'(?=\uD800)a',['a','\0']),
        (r'\N{ grinning face }',['😀','Vote']), (r'[\N{GRINNING FACE}]',['😀','Vote']),
        (r'\N{CJK UNIFIED IDEOGRAPHS 4E00}',['一','二']),
        (r'\N{PRIVATE USE AREA E000}',['\ue000','Vote']),
        (r'\b.{1}\b',['a','é','a\u0301','\u0301']),
        (r'(?U)\b.{1}\b',['a','é','a\u0301','\u0301']),
        (r'a\b\u0301',['a\u0301','a']), (r'(?U)a\b\u0301',['a\u0301','a']),
        (r'a*?(?i)b',['Ab','ab','aAb','b','B']),
        (r'a*?(?s).',['\n\n','a\n','A\n']),
        (r'a(?i)b|c',['C','AB','ab']),
        (r'(?i)a*?(?-i)b',['aAb','aab','aaB']),
        (r'a*(?i)b',['Ab','aAb','aaB']),
        ('a'*600,['a'*600,'a'*599]),
        (r'a{600}',['a'*600,'a'*599]),
        (r'a+',['a'*600,'a'*600+'b']),
        (r'a+?',['a'*600,'a'*600+'b']),
        ('a'*3000,['a'*3000,'a'*2999]),
        (r'a{3000}',['a'*3000,'a'*2999]),
        (r'(a)+',['a'*600,'a'*599+'b']),
        (r'(?:a)+',['a'*600,'a'*599+'b']),
        (r'(?i:(a))+',['aA'*300,'aA'*299+'b']),
        (r'(a)\1+',['a'*600,'a'*599+'b']),
        (r'\N{\u2003GRINNING FACE\u2003}',['😀','Vote']),
        ('\\N{\u2003GRINNING FACE\u2003}',['😀','Vote']),
        ('(?x)a\fb',['ab','a\fb']),('(?x)a\vb',['ab','a\vb']),
        ('(?x)a#comment\rb',['ab','a']),('(?x)a#comment\u2028b',['ab','a']),
        ('(?xd)a#comment\u2028b',['ab','a']),
        ('(?x)[a\fb]',['a','b','\f']),('(?x)[ a - z ]',['a','m','z','-',' ']),
        ('(?x)a#comment\u2028b',['a\u2028b','ab','a']),
        (r'(?x)a{1, 2}',['a','aa','aaa']), (r'(?x)a{ 1 , 2 }',['a','aa','aaa']),
        (r'(?x)(? i )a',['A','a','']), (r'(?x)[ a - z ]+',['hello','--','a-z']),
        (r'\08',['\0'+'8','8']), (r'\0',['\0','']), (r'\0123',['S','\n'+'3']),
        (r'a{2147483648}',['a','']), (r'a{1,2147483648}',['a','']),
        (r'\k<name>(?<name>a)',['aa','a']),
        (r'(?<名>a)',['a','']), (r'(?<_a>a)',['a','']), (r'(?<a_b>a)',['a','']),
        (r'\pQ',['a','']), (r'\p{^L}',['a','1']),
        (r'(?x)a{1 ,2}',['a','aa']), (r'(?x)a{1,2 }',['a','aa']),
        (r'(?x)a{1 2}',['a'*12,'a']), (r'(?x)a{1,2 3}',['a','a'*23]),
        (r'(?x)(?<a b>a)\k<ab>',['aa','a']),
        (r'(?x)\x{ 41}',['A','']), (r'(?x)\u00 41',['A','']),
        (r'(?x)[a - ]',['a','-',' ']), (r'[\d-a]',['1','a','-']),
    ]
    supplementary_inputs=['','a','😀','😀a','a😀','😀😀','😀a😀','a😀a','😀aa','𐐀𐐨','𐐨𐐀','A𐐀a𐐨','\u0301😀']
    for pattern in [
        r'(?<=😀)a',r'(?<!😀)a',r'(?<=😀{1,2})a',r'(?<=.)a',r'(?<!.)a',
        r'(?<=\p{L})a',r'(?<=\p{Cs})a',r'(?<=.{2})a',r'😀(?<=.)a',
        r'(?<=(.)(.))\1',r'(?<=(.)(.))\2',r'😀|(?<=(.)(.))\1',
        r'(?iu)(𐐀)\1',r'(?iu)(.)\1',r'(?iu)(𐐀)\1+',r'(?i)(𐐀)\1',
        r'\uDE00',r'[\uDE00]',r'\uD83D',r'[\uD83D]',r'\uDE00|a',
        r'a|\uDE00',r'😀|\uDE00',r'\uDE00|😀',r'.\uDE00',r'\X\uDE00',
        r'\uDE00\X',r'\x{D83D}\x{DE00}(?<=.)a',r'\x{D83D}\x{DE00}|(?<=.)a',
        r'(?=😀)(?<=(.)(.))\1',r'(?iu:i)|a',r'(?iu)ß|a',r'a|(?iu)i',
        r'(?iU)[a-z]',r'(?iu)[A-ÿ]',r'(?U)\w(?-U)\w',r'(?U)(?-U)\w',
        r'(?<=\x{1F600})a',r'(?<=\uD83D\uDE00)a',r'(?<=\uDE00)a',
        r'(?<=\x{D83D}\x{DE00})a',r'(?<=\N{GRINNING FACE})a',r'(?<=\p{L}😀)a',
        r'(?<=\p{L}|😀)a',r'(?<=\Q😀\E)a',r'(?<=.)😀',r'(?<=😀)(?<=.)a',
    ]:cases.append((pattern,supplementary_inputs))
    properties='L M N Z C P S Lu Ll Lt Lm Lo Mn Me Mc Nd Nl No Zs Zl Zp Cc Cf Co Cs Cn Pc Pd Ps Pe Pi Pf Po Sm Sc Sk So LC LD L1 all ASCII Alnum Alpha Blank Cntrl Digit Graph Lower Print Punct Space Upper XDigit javaLowerCase javaUpperCase javaTitleCase javaDigit javaDefined javaLetter javaLetterOrDigit javaJavaIdentifierStart javaJavaIdentifierPart javaUnicodeIdentifierStart javaUnicodeIdentifierPart javaIdentifierIgnorable javaSpaceChar javaWhitespace javaISOControl javaMirrored javaAlphabetic javaIdeographic'.split()
    properties += ['IsAlphabetic','IsLowercase','IsUppercase','IsTitlecase','IsWhite_Space','IsControl','IsPunctuation','IsHex_Digit','IsAssigned','IsNoncharacter_Code_Point','IsJoin_Control','IsWord','IsEmoji','IsEmoji_Presentation','IsEmoji_Modifier','IsEmoji_Modifier_Base','IsEmoji_Component','IsExtended_Pictographic','IsIdeographic','IsLatin','IsOld_Italic','IsLatn','sc=Greek','script=Grek','blk=Basic_Latin','block=Latin-1Supplement','gc=Lu','general_category=Ll','gc=javaLetter','gc=Lower','InGreek','InCyrillic_Supplementary','InCyrillic Supplement','IsGreek','Isgreek','IsGrEeK','isGreek','Inbasiclatin','inBasicLatin','Lowercase','lower','UPPER','IsLower','IsWHITESPACE','IsHexDigit','javaalphabetic','gc=lu','GC=Lu','sc=grek','script=old_italic']
    probes=['A','a','é','É','İ','ı','ſ','ß','1','١','\u0301','\u093e','\u200c','\u200d','\u0085','\u00a0','\u2003','\u001c','$','_','(', '⁈','∀','∊','Ⅰ','Ⅻ','一','𞓐','😀','\u0378','\ue000','\uffff','\U0010ffff']
    for name in properties:
        for flags in ['', '(?i)', '(?U)', '(?iU)']:
            cases.append((flags+r'\p{'+name+'}',probes))
    clusters=['','a','a\u0301','\r\n','\r\na','\u0301','\u0600a','\u0600a\u0301','\u1100\u1161\u11a8','\uac00\u11a8','\u0915\u094d\u0915','\u0915\u094d\u200d\u0915','👨\u200d👩\u200d👧','a\u200db','😀\u0301\u200d😀','🇦🇧','🇦🇧🇨','🇦🇧🇨🇩','👍🏽','\U000e0020','\u2028\u0301']
    for pattern in [r'\X',r'\X{2}',r'\X{3}',r'\b{g}',r'.\b{g}.',r'\X\b{g}\X',r'\b{g}\X\b{g}']:
        cases.append((pattern,clusters))
    # Structural combinations exercise captures, assertions, empty progress,
    # atom/group quantifiers and their continuations independently of Unicode
    # property coverage.
    for atom in ['a','.',r'[ab]',r'\R',r'\X','(a)','(a|b)','(?:a?)','(?:)',r'(?=(a))',r'(?!(a))',r'(?<=a)',r'(?<!a)',r'\1']:
        for quantifier in ['*','*?','*+','+','?','??','?+','{0}','{0,1}','{0,1}?','{0,1}+','{0,2}','{2}']:
            for tail in ['',r'a',r'\1']:
                cases.append((atom+quantifier+tail,['','a','b','aa','ab','ba','aaa','aba','\r\n','😀']))
    rows=[]
    def add(patterns,title,mode='Matches',screen='container',running=True,printing=True):
        rows.append({'patterns':patterns,'title':title,'mode':mode,'screen':screen,'running':running,'print':printing})
    for pattern,titles in cases:
        for title in titles:
            for mode in ['Matches','Contains']:add([pattern],title,mode)
    for pattern in ['[','(','a{2,1}',r'\p{Unknown}',r'\k<missing>',r'\x{110000}','(?<1x>a)',r'\q','(?z)a','*',r'\p{L',r'\x{20',r'\u123',r'[\q]',r'[\b]',r'\N{UNKNOWN NAME}',r'\N{GRINNING FACE',r'\p{}',r'\x{}',r'\uGGGG',r'\E']:
        add([pattern],'Vote')
    for patterns in [[],['^Vote$','^Shop$'],['^Shop$','^Vote$'],['^Vote$','^Vote$'],[None],[None,'^Vote$'],['^Vote$',None],['Never',None,'Vote']]:
        for title in ['Vote','Shop','Other']:
            for mode in ['Matches','Contains']:add(patterns,title,mode)
    for screen in ['container','other','null']:
        for running in [False,True]:
            for printing in [False,True]:
                for title in ['Vote','Other']:add(['^Vote$'],title,screen=screen,running=running,printing=printing)
    return rows

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True);fixture=out/'fixture';shutil.copytree(HERE/'gui-closer-source-fixture',fixture,dirs_exist_ok=True)
    source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/misc/ModuleGUICloser.kt'
    shutil.copyfile(source,fixture/source.name)
    inputs=vectors();(out/'inputs.json').write_text(json.dumps(inputs,ensure_ascii=True),encoding='utf8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';libs=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';stdlib=next(libs.glob('kotlin-stdlib-*.jar'));gson=next(libs.glob('gson-*.jar'));classes=out/'classes';classes.mkdir(exist_ok=True)
    commands=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf8')
        commands.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        return result
    result=run('compile',[str(java),'-cp',str(libs/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',str(stdlib)+';'+str(gson),'-d',str(classes),*map(str,sorted(fixture.glob('*.kt')))])
    if result.returncode==0:result=run('source',[str(java),'-cp',str(classes)+';'+str(stdlib)+';'+str(gson),'MainKt',str(out/'inputs.json')])
    if result.returncode==0:
        (out/'source-oracle.json').write_text(json.dumps(json.loads(result.stdout),ensure_ascii=True),encoding='utf8')
        result=run('layout',[str(java),'--add-opens=java.base/java.util.regex=ALL-UNNAMED','-cp',str(gson),str(fixture/'JdkRegexLayout.java'),str(out/'inputs.json')])
    if result.returncode==0:
        (out/'jdk-search-layout.json').write_text(json.dumps(json.loads(result.stdout),ensure_ascii=True),encoding='utf8')
        for item in [ROOT/'native/liquidbounce-behaviors/src/gui_closer.rs',fixture/'oracle.rs']:shutil.copyfile(item,out/item.name)
        (out/'lib.rs').write_text('pub mod gui_closer;\n#[cfg(test)]mod oracle;\n',encoding='utf8')
        shutil.copytree(ROOT/'native/liquidbounce-behaviors/vendor/java_regex',out/'vendor/java_regex',dirs_exist_ok=True)
        (out/'Cargo.toml').write_text('[package]\nname="gui-closer-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\njava_regex={path="vendor/java_regex"}\n',encoding='utf8')
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
        result=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')],env)
        if result.returncode==0:run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    snapshot=[*out.glob('*.rs'),*fixture.glob('*'),*out.glob('vendor/java_regex/**/*')]
    report={'passed':all(c['exitCode']==0 for c in commands),'vectors':len(inputs),'scope':'Byte-for-byte original ModuleGUICloser.kt, actual JDK25 Kotlin Regex, explicit event/module/Component/chat fixtures. Both modes, Java regex dialect features and compile errors, UTF16 and supplementary search selection, null-filter short circuit, screen/running/print gates, source chat call contents. Native Text styles and actual game integration remain separate checks.','sourceSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'unchangedSource':source.read_bytes()==(fixture/source.name).read_bytes(),'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(snapshot) if p.is_file()},'commands':commands}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
