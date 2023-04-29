import clone from 'clone';

/**
 * オブジェクトを全てのパラメータについて再帰的に遡って凍結します。
 * @param {Object} object オブジェクト
 */
export const deepFreeze = object => {
  Object.freeze(object);
  for (const key in object) {
    const value = object[key];
    if (
      !Object.hasOwnProperty.call(object, key) ||
      typeof value !== 'object' ||
      Object.isFrozen(value)
    ) {
      continue;
    }
    deepFreeze(value);
  }
};

export const camelToSnake = p =>
  p.replace(/([A-Z])/g, s => `_${s.charAt(0).toLowerCase()}`);

export const snakeToCamel = p => p.replace(/_./g, s => s.charAt(1).toUpperCase());
export const snakeToCamelHeadUpper = p => {
  const camel = snakeToCamel(p);
  return camel.substring(0, 1).toUpperCase() + camel.substring(1);
};
export const headLower = p => {
  return p.substring(0, 1).toLowerCase() + p.substring(1);
};
export const headUpper = p => {
  return p.substring(0, 1).toUpperCase() + p.substring(1);
};
export const replaceSnakeToCamel = object => {
  return replaceKeys(object, snakeToCamel);
};
const replaceHeadLower = object => {
  return replaceKeys(object, headLower);
};
export const replaceKeys = (object, func) => {
  if (typeof object !== 'object') {
    return object;
  }
  const replacedObj = {};
  for (const key in object) {
    const value = object[key];
    if (!Object.hasOwnProperty.call(object, key)) {
      continue;
    }
    replacedObj[func(key)] = value !== null ? replaceKeys(value, func) : null;
  }
  return replacedObj;
};

// 数値の0以外のnull,空文字はundefinedに変換する
export const objectConvUndefined = obj => {
  const o = clone(obj);
  for (const key of Object.keys(o)) {
    if (!o[key] || o[key] === 0) {
      o[key] = undefined;
    }
  }
  return o;
};
export const objectFilter = (
  obj,
  params,
  convSnakeCase = false,
  convCamelCase = false,
) => {
  if (params && params.length > 0) {
    return Object.keys(obj)
      .filter(key => params.includes(key))
      .reduce(
        (o, key) =>
          Object.assign(o, {
            [convSnakeCase
              ? camelToSnake(key)
              : convCamelCase
              ? snakeToCamel(key)
              : key]: obj[key],
          }),
        {},
      );
  }
  return obj;
};

export const objectFilterKey = (obj, params, keyFunc) => {
  // paramが空なら全てのキーを対象とする
  // TODO 再帰的でないので、階層深い場合にも対応できるようにしたい
  if (!params || params.length === 0) {
    params = Object.keys(obj);
  }
  return Object.keys(obj)
    .filter(key => params.includes(key))
    .reduce(
      (o, key) => {
        if (keyFunc) {
          return Object.assign(o, {
            [keyFunc(key)]: obj[key],
          })
        } else {
          return Object.assign(o, {
            [key]: obj[key],
          })
        }
      },
      {},
    );
};
//valueに対して関数を適用し、フィルタリングする
export const objectFilterFunc = (obj, func) => {
  // paramが空なら全てのキーを対象とする
  // TODO 再帰的でないので、階層深い場合にも対応できるようにしたい
  return Object.keys(obj)
    .filter(key => func(obj[key]))
    .reduce(
      (o, key) =>
        Object.assign(o, {
          [key]: obj[key],
        }),
      {},
    );
};

export const range = (from, to) => {
  let ind = 0;
  const ret = [];
  while (ind <= to - from) {
    ret.push(from + ind);
    ind += 1;
  }
  return ret;
};

export const obj2Array = (obj) => {
  const retArray = [];
  if (!obj) {
    return retArray;
  } else {    
    for (const key in Object.keys(obj)) {
      retArray.push(obj[key]);
    }
    return retArray;
  }
}
export const cNull = str => str || '';
export const isBlank = str => str === undefined || str === null || String(str).trim() === '';

export const strIns = (str, idx, val) => str.slice(0, idx) + val + str.slice(idx);
export const cardConv = (str) => {
  return str.replace(/[ａ-ｚ０-９]/g, function(s) {
    return String.fromCharCode(s.charCodeAt(0) - 65248);
  }).replace(/[-ー―−‐―]/g, '');
}

export const objectifyByKeyParam = (fromObject, keyName='id') => {
  const retObj = {};
  if (!fromObject) {
    return retObj;
  } else if (Array.isArray(fromObject)) {
    for (const obj of fromObject) {
      retObj[obj[keyName]] = obj;
    }
    return retObj;
  } else {
    for (const key of Object.keys(fromObject)) {
      const obj = fromObject[key];
      retObj[obj[keyName]] = obj;
    }
    return retObj;
  }
}
// export default {
//   deepFreeze,
//   camelToSnake,
//   snakeToCamel,
//   snakeToCamelHeadUpper,
//   headLower,
//   headUpper,
//   replaceKeys,
//   objectFilter,
//   objectFilterKey,
//   objectFilterFunc,
//   objectConvUndefined,
//   replaceSnakeToCamel,
//   replaceHeadLower,
//   range,
//   cNull,
//   isBlank,
//   strIns,
//   cardConv,
//   obj2Array,
// };
