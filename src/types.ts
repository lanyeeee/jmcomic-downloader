import {CategoryRespData, CategorySubRespData} from "./bindings.ts";

export type AlbumInfo = {
    id: string;
    author: string;
    name: string;
    category: CategoryRespData;
    category_sub: CategorySubRespData;
}

