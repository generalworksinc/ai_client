// TODO blankToNullはデフォルトtrueにする→空文字入れたかったらDB側で制御

import messages from './messages.js';
import validators from './validators.js';
import { headLower, objectFilter } from '~/lib/common.js';
// import pageScroll from '~/services/pageScroll';
import clone from 'clone';

const KEY_FORM = Symbol('$form'); // Fieldが持つformのparam key(JSON.stringifyされた時ループに陥るためSymbol化)
const KEY_RANDOM = Symbol('$key')

const formatFromResponse = value => headLower(value);
// // const formatForRequest = (value) => value;
// const formatForRequest = null;

// vueのランダム値生成の再、同時刻に生成されてキーが一意にならないことを防ぐためのIndex値
let atomicKeyIndex = 0;
function randomKey() {
  // TODO 重複がある（特にリストで）
  atomicKeyIndex = (atomicKeyIndex + 1) % 1000000000000;
  return Math.random() + atomicKeyIndex;
}

// class Field {
//   constructor(val, fieldName, validateWrapped, options) {
//     const self = this;

//     this.val = val;
//     this.fieldName = fieldName;
//     this.validator = {};
//     this.validateWrapped = validateWrapped;
//     this.messageText = {};
//     this.options = options || {};
//     if (this.options.blankToNull === undefined) {
//       this.options.blankToNull = true;
//     }

//     if (validateWrapped !== null && validateWrapped !== undefined) {
//       Object.keys(validateWrapped).forEach(key => {
//         if (key.startsWith('$validateParam_')) {
//           return;
//         }
//         this.validator[key] = validateWrapped[key];
//       });
//     }
//   }

//   message(obj) {
//     const messages = [];
//     if (obj.val) {
//       Object.keys(obj.val).forEach(key => {
//         if (!key.startsWith('$') && obj.val[key] === false) {
//           messages.push(this.getMessageText(key));
//         }
//       });
//     }
//     return messages.join('<br>');
//   }

//   getMessageText(key) {
//     if (!this.messageText[key]) {
//       let message = '';
//       // validatorのkeyは、'<validatorName>.<index>' という文字列になってるので、メッセージ用にkeyを取り出す
//       const messagesKey = key.split('.')[0];
//       message = messages[messagesKey]
//         ? messages[messagesKey].replace('{param}', this.fieldName)
//         : '';

//       const params = this.validateWrapped[`$validateParam_${key}`];
//       for (const ind in params) {
//         message = message.replace(`{${ind}}`, params[ind]);
//       }
//       // compare系のvaliationメッセージに対応(paramComparedと記述)
//       if (
//         message.indexOf('{paramCompared}') >= 0 &&
//         params[1] &&
//         this[FORM_KEY]
//       ) {
//         let validationObj = this[FORM_KEY];
//         params[1].split('.').forEach(param => {
//           validationObj = validationObj[param];
//         });
//         message = message.replace('{paramCompared}', validationObj.fieldName);
//       }
//       this.messageText[key] = message;
//     }
//     return this.messageText[key];
//   }

//   setParentForm(form) {
//     this[FORM_KEY] = form;
//   }
// }
// class ListField {
//   constructor(obj, defaultLength) {
//     this.obj = obj;
//     // vueの:keyを自動的に採番してくれるフィールドを準備する
//     this.obj.$key = F(randomKey(), '$key');
//     this.list = Array(defaultLength).fill(obj);
//   }

//   get() {
//     return this.list;
//   }

//   push(obj) {
//     // vueの:keyを自動的に採番してくれるフィールドを準備する
//     obj.$key = F(randomKey(), '$key');
//     this.list.push(obj);
//   }
// }
export class Form {
  constructor(m) {
    const model = clone(m);
    // 全てのフィールドに、Form自体にアクセスできるような参照を追加する
    Object.keys(model).forEach(key => {
      const obj = model[key];
      obj[KEY_FORM] = this;
      this[key] = obj;
    });
    // vueの:keyを自動的に採番してくれるフィールドを準備する
    this[KEY_RANDOM] = {value: randomKey(), name: '$key'};
  }

  /**
   * objectをダイレクトにForm中のフィールドにセットする。
   * snake_caseで渡しても、camelCaseに変換されます。
   *
   * obj: セットしたいObject
   * keyAndFunc: key: Field名 value:処理関数 ※特に処理がない場合、フィールドに値をセットするだけ
   */
  setData(obj, keyAndFunc) {
    console.log('obj:', obj, 'keyAndFunc:', keyAndFunc);
    Object.keys(obj).forEach(key => {
      const formatedkey = formatFromResponse(key);
      // デフォルトの処理（フィールドに値をセット）
      let setFunc = value => {
        if (
          this.hasOwnProperty(formatedkey)
          //  && this[formatedkey] instanceof Field
        ) {
          this[formatedkey].value = value;
        }
      };
      if (keyAndFunc) {
        if (keyAndFunc.hasOwnProperty(formatedkey)) {
          setFunc = keyAndFunc[formatedkey];
        } else if (keyAndFunc.hasOwnProperty(key)) {
          setFunc = keyAndFunc[key];
        }
      }
      // 処理を実行
      setFunc(obj[key]);
    });
  }

  /**
   * オブジェクトを文字列化したJsonで返す
   */
  getValueJsonStr({ keys = [], exceptKeys = [] }) {
    return JSON.stringify(this.getValueJson({ keys, exceptKeys }));
  }

  // /**
  //  * オブジェクトを文字列化したJsonで返す
  //  */
  // getValueJsonValueStr({ keys = [], exceptKeys = [] }) {
  //   const obj = this.getValueJson({ keys, exceptKeys });
  //   Object.keys(obj).forEach(key => {
  //     if (typeof obj[key] === 'object') obj[key] = JSON.stringify(obj[key]);
  //   });
  //   return obj;
  // }

  /**
   * Jsonオブジェクトに変換してデータを返す
   */
  getJson(obj = {}) {
    return this.getValueJson(obj);
  }
  /**
   * Jsonオブジェクトに変換してデータを返す
   */
  getValueJson({ keys = [], exceptKeys = [], format = null }) {
    if (!keys || keys.length === 0) {
      // fillterの指定がない場合、全てのカラムを対象とする
      keys = Object.keys(this);
    }
    if (exceptKeys && exceptKeys.length > 0) {
      keys = keys.filter(key => exceptKeys.indexOf(key) < 0);
    }
    const formatFunc = format || null;
    const comvFilterKeys = formatFunc ? keys.map(key => formatFunc(key)) : keys;
    return extractData(objectFilter(this, comvFilterKeys));
  }

  /**
   * オブジェクトのvuelidate用のValidationオブジェクトを返す
   */
  getValidations() {
    return extractValidations(this);
  }

  startValid() {
    this.$startValid = true;
  }

  isErrorField(fieldName) {
    if (!this.$startValid) {
      return false;
    }
    let hasError = false;
    const obj = this[fieldName];
    const validatorList = obj.validate;
    
    // Validatorが登録されている時のみ、チェックを実行
    // listで、fieldNameChain==1の場合は、validationのフィールド名指定が誤っているのでerrorとする
    if (!validatorList) {
      return true;
    }
    
    for (const validator of validatorList) {
      let validFunc = null;
      let validMessage = ''
      let params = [];
      let validStr = '';
      if (typeof validator === 'string') {
        validStr = validator;
      } else {
        validStr = validator[0];
        params = validator.slice(1);
      }
      validFunc = validators[validStr];
      validMessage = messages[validStr];
      
      if (!validFunc(obj.value, ...params)) {
        console.log('validator:', obj.name, obj.value, validStr, params, validMessage, validFunc);
        console.log('error has occured.', obj);

        //メッセージの可変箇所に対応
        validMessage = validMessage.replace('{param}', validStr);
        for (const ind in params) {
          validMessage = validMessage.replace(`{${ind}}`, params[ind]);
        }

        obj.message = validMessage;
        hasError = true;
      }
    }
    obj.error = hasError;
    if (!hasError) {
      obj.message = '';
    }
    return obj.error;
  }

  groupIsValid(fieldNames) {
    this.startValid();
    let keys = fieldNames;
    // fieldNamesがしていない場合、全項目を対象とする
    if (!fieldNames || fieldNames.length === 0) {
        keys = Object.keys(this);
    }

    // groupValid($v_obj, ['lastName','firstName','lastNameKana','firstNameKana']) のように指定
    // リストや入れ子の項目をチェックする場合は、 'educationList.universityName'のように指定する
    let isValid = true;

    for (const key of keys) {
      const fieldNameChain = key.split('.');
      if (fieldNameChain.length === 1) {
        if (this.isErrorField(key)) {
          isValid = false;
        }
      // } else if (fieldNameChain.length === 2) {
      //   const { $each } = validateObj[fieldNameChain[0]];
      //   for (const key in $each) {
      //     if (!Number.isNaN(key) && $each[key]) {
      //       const $v_field = $each[key][fieldNameChain[1]];
      //       result &= chkValid($v_field);
      //     }
      //   }
      } else {
        // TODO 機能進捗優先で進めるため、2階層以上の入れ子リストの対応は未実装・保留中です。
        throw new Error('未実装です。');
      }
    }
    // if (!result) {
    //   // dom操作が完了するタイミングを考慮し0.02秒後に実行
    //   setTimeout(
    //     () =>
    //       pageScroll.scrollByClass({
    //         objClass: 'js_validate_error',
    //         regulation_px_pc: -100,
    //         regulation_px_sp: -50,
    //       }),
    //     20,
    //   );
    // }
    return isValid;
  }

  getErrorMessages(
    validateObj,
    fieldNames,
    template = '{fieldName}： {message}',
  ) {
    const errorMessages = [];
    fieldNames.forEach(fieldName => {
      if (validateObj[fieldName].val.$error) {
        errorMessages.push(
          template
            .replace('{fieldName}', this[fieldName].fieldName)
            .replace(
              '{message}',
              this[fieldName].message(validateObj[fieldName]),
            ),
        );
      }
    });
    return errorMessages;
  }

  
}

// const F = (val, fieldName, validator, options) =>
//   new Field(val, fieldName, validator, options);
// const L = (obj, defaultLength) => new ListField(obj, defaultLength);
// const V = (...validators) => {
//   if (validators.length === 0) {
//     return {};
//   }
//   // Nuxtでvuelidateのvalidation offになっていた場合、フロントのValidatationを行わない
//   if (process && process.env && process.env.VUELIDATE_DISABLED_FLG) {
//     return {};
//   }
//   validators = validators.map(validator => {
//     if (typeof validator === 'function') {
//       return validator();
//     }
//     return validator;
//   });
//   return Object.assign(...validators);
// };

// function isDict(v) {
//   if (
//     typeof v === 'object' &&
//     v !== null &&
//     !(v instanceof Array) &&
//     !(v instanceof Date)
//   ) {
//     return true;
//   }
//   return false;
// }
const extractData = formObj => {
  const valueObj = {};

  Object.keys(formObj).forEach(key => {
    if (key.startsWith('$')) {
      return;
    }
    const obj = formObj[key];
    let value = null;

    // if (obj instanceof ListField) {
    //   value = obj.get().map(x => extractData(x));
    // } else if (obj instanceof Field) {
    //   value = obj;
    // } else if (isDict(obj)) {
    //   value = obj;
    // } else if (obj) {
    //   throw new Error(`unknown field type. ${key}:${obj}`);
    // }
    if (obj.value) {
      value = obj.value
    }

    if (value !== null) {
      if (obj.type === Number) {
        try {
          valueObj[key] = Number(value);
        } catch (ex) {
          console.log('Number parse error. key:', key, 'value:', value);
          valueObj[key] = null;
        }
      } else {
        valueObj[key] = value;
      }
    }
  });
  return valueObj;
};

// const extractObject = (formObj, { format = null }) => {
//   const valueObj = {};
//   Object.keys(formObj).forEach(key => {
//     const obj = formObj[key];
//     let value = null;

//     // vueのv-forで使うkey用の値はObject化対象外とする
//     if (key === '$key') {
//       return;
//     }
//     if (obj instanceof ListField) {
//       value = obj.get().map(x => extractObject(x, {}));
//     } else if (obj instanceof Field) {
//       value = obj.val;
//       // Fieldに、空白だったらnullにするオプションが付いている場合の対応
//       if (obj.options.blankToNull && value === '') {
//         value = null;
//       }
//       if (obj.options.format) {
//         value = obj.options.format(value);
//       }
//     } else if (isDict(obj)) {
//       value = extractObject(obj, {});
//     } else if (obj) {
//       throw new Error('@extractObject unknown field type:', obj);
//     }

//     valueObj[format ? format(key) : key] = value;
//   });
//   return valueObj;
// };

// const extractValidations = formObj => {
//   const validatorObj = {};

//   Object.keys(formObj).forEach(key => {
//     const obj = formObj[key];
//     let validator = null;

//     if (obj instanceof ListField) {
//       validator = { $each: extractValidations(obj.obj) };
//       if (validator !== null && validator !== undefined) {
//         validatorObj[key] = validator;
//       }
//     } else if (obj instanceof Field) {
//       validator = obj.validator;
//       if (validator !== null && validator !== undefined) {
//         validatorObj[key] = { val: validator };
//       }
//     } else if (isDict(obj)) {
//       validator = extractValidations(obj);
//       if (validator !== null && validator !== undefined) {
//         validatorObj[key] = validator;
//       }
//     } else if (obj) {
//       throw new Error(`@extractValidation unknown field type:${obj}`);
//     }
//   });
//   return validatorObj;
// };

// const formUtil = {
//   newInstance: (formObj, defaultObj) => {
//     const retFormObj = clone(formObj);
//     if (defaultObj) {
//       Object.keys(defaultObj).forEach(key => {
//         const camelKey = commonJs.snakeToCamel(key);
//         if (retFormObj.hasOwnProperty(camelKey)) {
//           retFormObj[camelKey].val = defaultObj[key];
//         }
//       });
//     }
//     return retFormObj;
//   },
// };
const validatorMapForForm = {};
Object.keys(validators).forEach((validatorName) => {
  validatorMapForForm[validatorName] = (...params)=>[validatorName].concat(params);
});
//Formで指定するためのvalidatorをexportする
export const { maxLength } = validatorMapForForm;
export const { required } = validatorMapForForm;
export const { anyCondition } = validatorMapForForm;
export const { sameAs, isEmail } = validatorMapForForm;