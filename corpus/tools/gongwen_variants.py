"""Write 公文 specs of every common kind and length, and build them with `wenshu gongwen new`.

    python corpus/tools/gongwen_variants.py <wenshu-exe> corpus/files

The text is Wenshu's own, assembled from the sentence bank below, so the files are free to share.
Deterministic: the same seed gives the same documents. Lengths run from half a page to about four
pages; headings use the 公文 levels 一、（一）1.（1）, and sentences mix the marks the layout
patches care about (“” 《》 （） —— …… and digits).
"""
import json
import random
import subprocess
import sys
import tempfile
from pathlib import Path

KINDS = [
    ("通知", "关于做好{t}工作的通知", "各区人民政府，市政府各部门："),
    ("请示", "关于{t}有关事项的请示", "市人民政府："),
    ("报告", "关于{t}情况的报告", "市人民政府："),
    ("函", "关于商请支持{t}的函", "某某省某某厅："),
    ("批复", "关于{t}有关问题的批复", "某某区人民政府："),
    ("意见", "关于进一步推进{t}的实施意见", "各区人民政府，市政府各部门、各直属机构："),
    ("通报", "关于{t}检查情况的通报", "各有关单位："),
    ("决定", "关于表彰{t}先进集体和先进个人的决定", "各区人民政府，市政府各部门："),
    ("公告", "关于{t}的公告", ""),
    ("纪要", "{t}专题会议纪要", ""),
]
SLUGS = ["tongzhi", "qingshi", "baogao", "han", "pifu", "yijian", "tongbao", "jueding", "gonggao", "jiyao"]
TOPICS = ["办公软件国产化替代", "政务数据共享", "安全生产大检查", "城市更新试点", "冬季供暖保障",
          "营商环境优化", "电子公文交换", "基层减负", "防汛抗旱", "数字档案馆建设"]
HEADINGS = ["总体要求", "主要任务", "保障措施", "工作安排", "有关要求", "基本情况", "存在问题", "下一步打算"]
SENTENCES = [
    "各单位要高度重视，切实把思想和行动统一到决策部署上来。",
    "按照《党政机关公文格式》（GB/T 9704—2012）的要求，规范公文格式。",
    "到2027年底，全市{t}覆盖率达到95%以上，重点领域实现“全覆盖”。",
    "坚持问题导向——哪里有短板，就在哪里下功夫……确保取得实效。",
    "建立“周调度、月通报、季考核”工作机制，压实各方责任。",
    "对工作中发现的问题，要立行立改；一时难以整改的，要明确时限（原则上不超过3个月）。",
    "市财政安排专项资金1200万元，用于支持{t}相关项目建设。",
    "各区要于每月5日前将上月工作进展报送市{t}工作领导小组办公室。",
    "加强宣传引导，及时总结推广典型经验，营造良好氛围。",
    "严格落实“一岗双责”，主要负责同志要亲自抓、负总责。",
    "联系人：张某某，电话：0000-12345678，电子邮箱：test@example.gov.cn。",
    "本通知自印发之日起施行，有效期5年。",
]


def spec(i, rng):
    kind, title_fmt, recipients = KINDS[i % len(KINDS)]
    t = TOPICS[i % len(TOPICS)]
    body = ["为贯彻落实上级有关部署，扎实推进{t}工作，现就有关事项{k}如下。".format(t=t, k="通知" if kind == "通知" else "报告" if kind == "报告" else "明确")]
    sections = 1 + i % 4 * 2  # 1, 3, 5, 7 headed sections: half a page to about four pages
    for h in range(sections):
        body.append(f"{'一二三四五六七八'[h % 8]}、{HEADINGS[(h + i) % len(HEADINGS)]}")
        for sub in range(rng.randint(1, 3)):
            body.append(f"（{'一二三四'[sub]}）" + rng.choice(SENTENCES).format(t=t))
            for _ in range(rng.randint(1, 3)):
                body.append(rng.choice(SENTENCES).format(t=t))
    return {
        "organ_mark": "某某市人民政府文件" if i % 2 == 0 else "某某市人民政府办公室文件",
        "number": f"某政{'办' if i % 2 else ''}发〔2026〕{10 + i}号",
        "title": title_fmt.format(t=t),
        "recipients": recipients,
        "body": body,
        "signer": "某某市人民政府" if i % 2 == 0 else "某某市人民政府办公室",
        "date": f"2026年{1 + i % 12}月{1 + (i * 7) % 28}日",
    }


def main(exe, out, n=10):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    rng = random.Random(9704)
    with tempfile.TemporaryDirectory() as tmp:
        for i in range(n):
            s = spec(i, rng)
            kind = SLUGS[i % len(SLUGS)]
            js = Path(tmp) / f"{i}.json"
            js.write_text(json.dumps(s, ensure_ascii=False, indent=1), encoding="utf-8")
            target = out / f"gen-gongwen-{i + 1:02d}-{kind}.docx"
            subprocess.run([exe, "gongwen", "new", str(js), str(target)], check=True, capture_output=True)
    print(f"{n} 公文 in {out}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
