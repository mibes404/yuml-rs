#!/usr/bin/env bash
dot -Tsvg <<EOM
digraph G {
  graph [ bgcolor=transparent, fontname=Helvetica ]
  node [ shape=none, margin=0, color=black, fontcolor=black, fontname=Helvetica ]
  edge [ color=black, fontcolor=black, fontname=Helvetica ]
    ranksep = 0.5
    rankdir = TB
    A9 -> A10 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A10 [shape="doublecircle" , margin="0,0" , label="" , style="" , height=0.3 , width=0.3 , ]
    A6 -> A9 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A8 -> A6:f2:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A7 -> A8 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A4 -> A7 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A9 [shape="rectangle" , margin="0.20,0.05" , label="Pour Water" , style="rounded" , height=0.5 , width=0 , fontsize=10 , ]
    A8 [shape="rectangle" , margin="0.20,0.05" , label="Add Milk" , style="rounded" , height=0.5 , width=0 , fontsize=10 , ]
    A7 [shape="rectangle" , margin="0.20,0.05" , label="Add Tea Bag" , style="rounded" , height=0.5 , width=0 , fontsize=10 , ]
    A5 -> A6:f1:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A4 -> A5 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A2 -> A4:f2:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="[kettle full]" , labeldistance=1 , fontsize=10 , ]
    A6 [shape="record" , margin="0,0" , label="<f1>|<f2>" , style="filled" , height=0.05 , width=0.5 , fontsize=1 , penwidth=4 , ]
    A5 [shape="rectangle" , margin="0.20,0.05" , label="Boil Kettle" , style="rounded" , height=0.5 , width=0 , fontsize=10 , ]
    A3 -> A4:f1:n [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A2 -> A3 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="[kettle empty]" , labeldistance=1 , fontsize=10 , ]
    A1 -> A2 [shape="edge" , dir="both" , style="solid" , arrowtail="none" , arrowhead="vee" , label="" , labeldistance=1 , fontsize=10 , ]
    A4 [shape="record" , margin="0,0" , label="<f1>|<f2>" , style="filled" , height=0.05 , width=0.5 , fontsize=1 , penwidth=4 , ]
    A3 [shape="rectangle" , margin="0.20,0.05" , label="Fill Kettle" , style="rounded" , height=0.5 , width=0 , fontsize=10 , ]
    A2 [shape="diamond" , margin="0,0" , label="" , style="" , height=0.5 , width=0.5 , fontsize=0 , ]
    A1 [shape="circle" , margin="0,0" , label="" , style="" , height=0.3 , width=0.3 , ]
}
EOM
