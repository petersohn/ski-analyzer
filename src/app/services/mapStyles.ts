import { Style, Stroke, Fill, Circle, } from "ol/style";

export type PisteStyleBase = {
  line: Style;
  area: Style;
};

export type PisteStyle = {
  unselected: PisteStyleBase;
  selected: PisteStyleBase;
};

export type PisteStyles = {
  [difficulty: string]: PisteStyle;
};

export const liftStyle = new Style({
  stroke: new Stroke({
    color: "#333",
    width: 2,
  }),
});

export const liftStyleSelected = new Style({
  stroke: new Stroke({
    color: "#333",
    width: 3,
  }),
});

export const stationStyle = new Style({
  image: new Circle({
    radius: 3,
    fill: new Fill({
      color: "#000",
    }),
  }),
});

export const pisteStyles: PisteStyles = {
  Novice: {
    unselected: {
      line: new Style({
        stroke: new Stroke({
          color: "#0a0",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0a04",
        }),
      }),
    },
    selected: {
      line: new Style({
        stroke: new Stroke({
          color: "#0a0",
          width: 3,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0a08",
        }),
      }),
    },
  },
  Easy: {
    unselected: {
      line: new Style({
        stroke: new Stroke({
          color: "#00f",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#00f4",
        }),
      }),
    },
    selected: {
      line: new Style({
        stroke: new Stroke({
          color: "#00f",
          width: 3,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#00f8",
        }),
      }),
    },
  },
  Intermediate: {
    unselected: {
      line: new Style({
        stroke: new Stroke({
          color: "#f00",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#f004",
        }),
      }),
    },
    selected: {
      line: new Style({
        stroke: new Stroke({
          color: "#f00",
          width: 3,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#f008",
        }),
      }),
    },
  },
  Advanced: {
    unselected: {
      line: new Style({
        stroke: new Stroke({
          color: "#000",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0004",
        }),
      }),
    },
    selected: {
      line: new Style({
        stroke: new Stroke({
          color: "#000",
          width: 3,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0008",
        }),
      }),
    },
  },
  Expert: {
    unselected: {
      line: new Style({
        stroke: new Stroke({
          color: "#000",
          width: 2,
          lineDash: [6, 4],
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0004",
        }),
      }),
    },
    selected: {
      line: new Style({
        stroke: new Stroke({
          color: "#000",
          width: 3,
          lineDash: [6, 4],
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0008",
        }),
      }),
    },
  },
  Freeride: {
    unselected: {
      line: new Style({
        stroke: new Stroke({
          color: "#f60",
          width: 2,
          lineDash: [6, 4],
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#f604",
        }),
      }),
    },
    selected: {
      line: new Style({
        stroke: new Stroke({
          color: "#f60",
          width: 3,
          lineDash: [6, 4],
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#f608",
        }),
      }),
    },
  },
  Unknown: {
    unselected: {
      line: new Style({
        stroke: new Stroke({
          color: "#888",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#8884",
        }),
      }),
    },
    selected: {
      line: new Style({
        stroke: new Stroke({
          color: "#888",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#8888",
        }),
      }),
    },
  },
};

export const routeStyles: { [key: string]: Style } = {
  Unknown: new Style({
    stroke: new Stroke({
      color: "#f0f",
      width: 2,
    }),
  }),
  UseLift: new Style({
    stroke: new Stroke({
      color: "#a00",
      width: 2,
    }),
  }),
};
