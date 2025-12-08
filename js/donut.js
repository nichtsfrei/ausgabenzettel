// Stolen from: https://github.com/finom/donutjs/blob/master/donut.js
(function (root, factory) {
  if (typeof define === "function" && define.amd) {
    define(factory);
  } else if (typeof module === "object" && module.exports) {
    module.exports = factory();
  } else {
    root.donut = factory();
  }
})(this, function () {
  var doc = document,
    M = Math,
    donutData = {},
    dataIndex = 0,
    donut = function (options) {
      var div = doc.createElement("div"),
        size = options.size || 100,
        data = options.data || [{ value: 1 }],
        weight = options.weight || 50,
        el = options.el,
        r = size / 2,
        PI = M.PI,
        sin = M.sin,
        cos = M.cos,
        sum = 0,
        i,
        value,
        arc,
        setAttribute = function (el, o) {
          for (j in o) {
            el.setAttribute(j, o[j]);
          }
        };

      for (i = 0; i < data.length; i++) {
        sum += data[i].value;
      }

      div.className = "donut";
      div.style.width = div.style.height = size + "px";

      var NS = "http://www.w3.org/2000/svg",
        svg = doc.createElementNS(NS, "svg"),
        startAngle = -PI / 2,
        arcRadius = r - weight / 2;

      svg.setAttribute("height", size + "px");
      svg.setAttribute("width", size + "px");

      div.appendChild(svg);

      for (i = 0; i < data.length; i++) {
        value = data[i].value / sum;
        value = value === 1 ? 0.9999 : value;
        arc = doc.createElementNS(NS, "path");

        var segmentAngle = value * PI * 2,
          endAngle = segmentAngle + startAngle,
          largeArc = (endAngle - startAngle) % (PI * 2) > PI ? 1 : 0,
          startX = r + cos(startAngle) * arcRadius,
          startY = r + sin(startAngle) * arcRadius,
          endX = r + cos(endAngle) * arcRadius,
          endY = r + sin(endAngle) * arcRadius;

        startAngle = endAngle;

        setAttribute(arc, {
          d: [
            "M",
            startX,
            startY,
            "A",
            arcRadius,
            arcRadius,
            0,
            largeArc,
            1,
            endX,
            endY,
          ].join(" "),
          stroke: data[i].color,
          "stroke-width": weight,
          fill: "none",
          "data-name": data[i].name,
          class: "donut-arc",
        });
        donut.data(arc, data[i]);
        //TODO: add agenda
        svg.appendChild(arc);
      }

      if (el) {
        el.appendChild(div);
      }

      return div;
    };

  donut.data = function (arc, data) {
    if (typeof data === "undefined") {
      return donutData[arc._DONUT];
    } else {
      donutData[(arc._DONUT = arc._DONUT || ++dataIndex)] = data;
      return arc;
    }
  };

  return donut;
});
