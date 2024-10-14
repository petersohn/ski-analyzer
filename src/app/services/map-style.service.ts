import { Injectable } from "@angular/core";
import { Style, Stroke, Fill, Circle } from "ol/style";
import { Options as StrokeOptions } from "ol/style/Stroke";
import { lazy } from "@/utils/lazy";

export type SelectableStyle = {
  unselected: Style;
  selected: Style;
};

export type PisteStyle = {
  line: SelectableStyle;
  area: SelectableStyle;
};

export type PisteStyles = {
  [difficulty: string]: PisteStyle;
};

export type RouteStyle = {
  line: SelectableStyle;
  node: SelectableStyle;
};

export type RouteStyles = {
  [type: string]: RouteStyle;
};

@Injectable({ providedIn: "root" })
export class MapStyleService {
  public liftStyle = lazy<SelectableStyle>(() => {
    const color = "#333";
    return {
      unselected: new Style({
        stroke: new Stroke({
          color,
          width: 2,
        }),
      }),
      selected: new Style({
        stroke: new Stroke({
          color,
          width: 3,
        }),
      }),
    };
  });

  public stationStyle = lazy(
    () =>
      new Style({
        image: new Circle({
          radius: 3,
          fill: new Fill({
            color: "#000",
          }),
        }),
      }),
  );

  public pisteStyles = lazy(() => {
    const colors: { [difficulty: string]: string } = {
      Novice: "#0a0",
      Easy: "#00f",
      Intermediate: "#f00",
      Advanced: "#000",
      Expert: "#000",
      Freeride: "#f60",
      Unknown: "888",
    };
    const lineProperties: { [difficulty: string]: StrokeOptions } = {
      Expert: { lineDash: [6, 4] },
      Freeride: { lineDash: [6, 4] },
    };

    const result: PisteStyles = {};

    for (const difficulty in colors) {
      result[difficulty] = {
        line: {
          unselected: new Style({
            stroke: new Stroke({
              color: colors[difficulty],
              width: 2,
              ...(lineProperties[difficulty] ?? {}),
            }),
          }),
          selected: new Style({
            stroke: new Stroke({
              color: colors[difficulty],
              width: 3,
              ...(lineProperties[difficulty] ?? {}),
            }),
          })
        },
        area: {
          unselected: new Style({
            fill: new Fill({
              color: colors[difficulty] + "4",
            }),
          }),
          selected: new Style({
            fill: new Fill({
              color: colors[difficulty] + "8",
            }),
          })
        },
      };
    }

    return result;
  });

  public routeStyles = lazy(() => {
    const colors: { [type: string]: string } = {
      Unknown: "#f0f",
      UseLift: "#a00",
    };

    const result: RouteStyles = {};
    for (const type in colors) {
      const color = colors[type];
      result[type] = {
        line: {
          unselected: new Style({
            stroke: new Stroke({
              color,
              width: 2,
            }),
          }),
          selected: new Style({
            stroke: new Stroke({
              color,
              width: 3,
            }),
          }),
        },
        node: {
          unselected: new Style({}),
          selected: new Style({
            image: new Circle({
              radius: 4,
              fill: new Fill({
                color,
              }),
            }),
          }),
        },
      };
    }
    return result;
  });
}
