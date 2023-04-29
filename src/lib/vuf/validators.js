import v8n from "v8n";
//
// import {
//   maxValue as maxValue_tmp,
//   minValue as minValue_tmp,
//   and as and_tmp,
//   or as or_tmp,
//   url as url_tmp,
//   sameAs as sameAs_tmp,
//   requiredIf as requiredIf_tmp,
//   requiredUnless as requiredUnless_tmp,
//   required as required_tmp,
//   minLength as minLength_tmp,
//   maxLength as maxLength_tmp,
//   macAddress as macAddress_tmp,
//   ipAddress as ipAddress_tmp,
//   // email as email_tmp,
//   between as between_tmp,
//   numeric as numeric_tmp,
//   alphaNum as alphaNum_tmp,
//   alpha as alpha_tmp,
// } from 'vuelidate/lib/validators';
// import services from '~/services/index';

// const { cNull } = services.common;
// const TEXT_LENGTH = 5000; // TableのTEXTカラムに許容する文字数のデフォルト値。個別に調整したい場合はmaxLengthを使う
// const emailRegex = /^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$/;

// //
// const comparation = (
//   validationObj,
//   thisFieldValue,
//   fieldName,
//   comparationFunc,
// ) => {
//   fieldName.split('.').forEach(param => {
//     validationObj = validationObj[param];
//   });
//   // nullもしくは空の場合は比較しない
//   return (
//     cNull(thisFieldValue) === '' ||
//     cNull(validationObj.val) === '' ||
//     comparationFunc(thisFieldValue, validationObj.val)
//   );
// };
// const originalValidators = {
//   maxValue_tmp,
//   minValue_tmp,
//   and_tmp,
//   or_tmp,
//   url_tmp,
//   // positiveInteger
//   positiveInteger_tmp(value) {
//     return cNull(value) === '' || (numeric_tmp(value) && parseInt(value) >= 0);
//   },
//   // sameAs_tmp,
//   sameAs_tmp: fieldName =>
//     function(thisFieldValue) {
//       const func = (a, b) =>
//         // それ以外は文字列（trim）比較
//         String(a).trim() === String(b).trim();
//       return comparation(this, thisFieldValue, fieldName, func);
//     },
//   // greater than (>),
//   gt_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => isNaN(a) || isNaN(b) || parseFloat(a) > parseFloat(b),
//       );
//     },
//   // greater equal (>=),
//   ge_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => isNaN(a) || isNaN(b) || parseFloat(a) >= parseFloat(b),
//       );
//     },
//   // less than (<),
//   lt_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => isNaN(a) || isNaN(b) || parseFloat(a) < parseFloat(b),
//       );
//     },
//   // less equal (<=),
//   le_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => isNaN(a) || isNaN(b) || parseFloat(a) <= parseFloat(b),
//       );
//     },
//   // /////comparation strings
//   // greater than string (>),
//   gtStr_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => cNull(a) === '' || cNull(b) === '' || a > b,
//       );
//     },
//   // greater equal string (>=),
//   geStr_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => cNull(a) === '' || cNull(b) === '' || a >= b,
//       );
//     },
//   // less than string (<),
//   ltStr_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => cNull(a) === '' || cNull(b) === '' || a < b,
//       );
//     },
//   // less equal string (<=),
//   leStr_tmp: fieldName =>
//     function(thisFieldValue, formParam) {
//       return comparation(
//         this,
//         thisFieldValue,
//         fieldName,
//         (a, b) => cNull(a) === '' || cNull(b) === '' || a <= b,
//       );
//     },
//   // vueコンポーネント内にmethods,computed等で記述された、任意の条件がtrueの時のみチェックする
//   requiredIf_tmp: conditionParam =>
//     function(thisFieldValue) {
//       if (
//         !Array.isArray(conditionParam) &&
//         typeof this[conditionParam] !== 'function'
//       ) {
//         throw new Error(
//           `validation error. 指定された条件:${conditionParam}は、vueコンポーネント内で定義されていません。`,
//         );
//       }
//       // 複数の条件指定されていた場合、全ての条件がtrueの場合にrequiredチェックを行う
//       if (Array.isArray(conditionParam)) {
//         return (
//           conditionParam.some(f => !this[f]()) || required_tmp(thisFieldValue)
//         );
//       }
//       return !this[conditionParam]() || required_tmp(thisFieldValue);
//     },
//   // vueComponent内で定義されたメソッドを引数に受け取る、そのメソッド内で入力値のチェックを行う
//   anyCondition_tmp: conditionParam =>
//     function(thisFieldValue) {
//       const conditions = Array.isArray(conditionParam)
//         ? conditionParam
//         : [conditionParam];

//       conditions.forEach(condition => {
//         if (!this[condition] || typeof this[condition] !== 'function') {
//           throw new Error(
//             `validation error. 指定された条件:${conditionParam}は、vueコンポーネント内で定義されていません。`,
//           );
//         }
//       });

//       // 値が空であればチェックを行わない
//       if (cNull(thisFieldValue) === '') {
//         return true;
//       }
//       // 全ての条件についてチェックを行う
//       return conditions.every(f => this[f]());
//     },
//   // vueComponent内で定義されたメソッドを引数に受け取る、そのメソッド内で入力値のチェックを行う
//   anyCondition_tmp: conditionParam =>
//     function(thisFieldValue) {
//       if (typeof this[conditionParam] !== 'function') {
//         throw new Error(
//           `validation error. 指定された条件:${conditionParam}は、vueコンポーネント内で定義されていません。`,
//         );
//       }
//       return this[conditionParam]() || cNull(thisFieldValue) === '';
//     },
//   // vueコンポーネント内にmethods,computed等で記述された、任意の条件がfalseの時のみチェックする
//   requiredUnless_tmp: conditionParam =>
//     function(thisFieldValue) {
//       if (
//         !Array.isArray(conditionParam) &&
//         typeof this[conditionParam] !== 'function'
//       ) {
//         throw new Error(
//           `validation error. 指定された条件:${conditionParam}は、vueコンポーネント内で定義されていません。`,
//         );
//       }
//       // 複数の条件指定されていた場合、全ての条件がfalseの場合にrequiredチェックを行う
//       if (Array.isArray(conditionParam)) {
//         return (
//           conditionParam.some(f => this[f]()) || required_tmp(thisFieldValue)
//         );
//       }
//       return this[conditionParam]() || required_tmp(thisFieldValue);
//     },
//   // requiredUnless_tmp,
//   // requiredIf_tmp,
//   required_tmp,
//   minLength_tmp,
//   maxLength_tmp,
//   macAddress_tmp,
//   ipAddress_tmp,
//   email_tmp(value) {
//     return emailRegex.test(value);
//   },
//   between_tmp,
//   numeric_tmp,
//   alphaNum_tmp,
//   alpha_tmp,
//   tel_tmp(value) {
//     if (value !== null && value !== '') {
//       return !!value.match(/^[0-9]+[-0-9]+[0-9]$/);
//     }
//     return true;
//   },
// };
// const validators = {};

// Object.keys(originalValidators).forEach(key_tmp => {
//   const f = originalValidators[key_tmp];
//   const key = key_tmp.split('_')[0]; // _tmpを外す

//   // パラメータを取得できるように$validateParam_付きのキーに配列で保持する
//   validators[key] = (...params) => {
//     const obj = {};
//     // 同じ種類のvalidationを複数登録できるように、ランダムなsuffixを付ける
//     const randomSuffix = String(Math.random()).substr(2);
//     if (params.length > 0) {
//       obj[`${key}.${randomSuffix}`] = f(...params); // 引数有りのvalidator
//     } else {
//       obj[`${key}.${randomSuffix}`] = f; // 引数無しのvalidator
//     }
//     obj[`$validateParam_${`${key}.${randomSuffix}`}`] = params;
//     return obj;
//   };
// });

// export const { maxValue } = validators;
// export const { minValue } = validators;
// export const { and } = validators;
// export const { or } = validators;
// export const { url } = validators;
// export const { sameAs } = validators;
// export const { gt } = validators;
// export const { ge } = validators;
// export const { lt } = validators;
// export const { le } = validators;
// export const { gtStr } = validators;
// export const { geStr } = validators;
// export const { ltStr } = validators;
// export const { leStr } = validators;
// export const { requiredUnless } = validators;
// export const { requiredIf } = validators;
// export const { validateUnless } = validators;
// export const { validateIf } = validators;
// export const requiredTos = validators.required; // chkロジックはrequiredを使いまわし、エラー文言だけオリジナルで指定
// export const { required } = validators;
// export const { minLength } = validators;
// export const { maxLength } = validators;
// export const textLength = validators.maxLength(TEXT_LENGTH);
// export const { macAddress } = validators;
// export const { ipAddress } = validators;
// export const { email } = validators;
// export const { between } = validators;
// export const { numeric } = validators;
// export const { positiveInteger } = validators;
// export const { alphaNum } = validators;
// export const { alpha } = validators;
// export const { tel } = validators;
// export const { anyCondition } = validators;

export default {
  required: (value) => {
    const result = v8n().string().minLength(1).test(String(value));
    return result;
  },
  maxLength: (value, num) => {
    return v8n().maxLength(num).test(String(value));
  },
  anyCondition: (value, param) => {
    throw Error("anyConditionは 未実装です"); //TODO
    return false;
  },
  sameAs: (value, param) => {
    throw Error("sameAsは 未実装です"); //TODO
    return false;
  },
  integer: (value) => {
    try {
      return v8n().integer().test(Number(value));
    } catch (ex) {
      return false;
    }
},
  positiveInteger: (value) => {
    // console.log('check positiveInteger value:', value);
    try {
      const result = v8n().integer().positive().test(Number(value));
      return result;
    } catch (ex) {
      console.log('exception: ex:', ex)
      return false;
    }
  },
  isEmail: (value) => {
    return v8n().not.null() // 値がnullじゃないか
      .string() // 文字列
      .minLength(5) // a@b.c を想定して最低5文字
      .pattern(/[^\s@]+@[^\s@]+\.[^\s@]+/) // eメール用の正規表現
      .test(value); 
  }
}
