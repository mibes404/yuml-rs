#!/usr/bin/env bash
dot -Tpng -o test.png <<EOM
digraph G {
  graph [ bgcolor=transparent, fontname=Helvetica ]
  node [ shape=none, margin=0, color=black, fontcolor=black, fontname=Helvetica ]
  edge [ color=black, fontcolor=black, fontname=Helvetica ]
    ranksep = 0.7
    rankdir = TB
    A1 [shape="note" , margin="0.20,0.05" , label="You can stick notes on diagrams
too!\\{bg:cornsilk\\}" , style="filled" , fillcolor="cornsilk" , fontcolor="black" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A2 [shape="rectangle" , margin="0.20,0.05" , label="Customer" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A3 [shape="rectangle" , margin="0.20,0.05" , label="Order" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A2 -> A3 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="odiamond" , arrowhead="vee" , taillabel="1" , headlabel="rders 0..*>" , labeldistance=2 , fontsize=10 , ]
    A4 [shape="rectangle" , margin="0.20,0.05" , label="LineItem" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A3 -> A4 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="diamond" , arrowhead="vee" , taillabel="*" , headlabel=">" , labeldistance=2 , fontsize=10 , ]
    A5 [shape="rectangle" , margin="0.20,0.05" , label="DeliveryMethod" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A3 -> A5 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="vee" , taillabel="" , headlabel=">" , labeldistance=2 , fontsize=10 , ]
    A6 [fontsize=10,label=<<TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0" CELLPADDING="9" ><TR><TD>Product</TD></TR><TR><TD>EAN_Code</TD></TR><TR><TD>promo_price()</TD></TR></TABLE>>]
    A3 -> A6 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="none" , arrowhead="vee" , taillabel="*" , headlabel=">" , labeldistance=2 , fontsize=10 , ]
    A7 [shape="rectangle" , margin="0.20,0.05" , label="Category" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A7 -> A6 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="vee" , arrowhead="vee" , taillabel="" , headlabel="" , labeldistance=2 , fontsize=10 , ]
    A8 [shape="rectangle" , margin="0.20,0.05" , label="National" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A5 -> A8 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="empty" , arrowhead="none" , labeldistance=2 , fontsize=10 , ]
    A9 [shape="rectangle" , margin="0.20,0.05" , label="International" , style="" , arrowtail="none" , arrowhead="none" , height=0.5 , fontsize=10 , ]
    A5 -> A9 [shape="edge" , label="" , style="solid" , dir="both" , arrowtail="empty" , arrowhead="none" , labeldistance=2 , fontsize=10 , ]
}
EOM
