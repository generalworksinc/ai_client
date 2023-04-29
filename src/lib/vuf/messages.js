// vuelidateを利用したバリデーションの出力名です。 項目名の箇所に、※{param}と記載してください。
// isUnique: '{param}は、既に登録されています。',
export default {
  // none	Requires non-empty data. Checks for empty arrays and strings containing only whitespaces.
  required: '※必須入力項目です。',
  // locator *	Requires non-empty data only if provided property or predicate is true.
  requiredIf: '※必須です。',
  // locator *	Requires non-empty data only if provided property or predicate is false.
  requiredUnless: '※{param}は、必須です。',
  // min length	Requires the input to have a minimum specified length, inclusive. Works with arrays.
  minLength: '※{0}文字以上で入力してください。',
  // max length	Requires the input to have a maximum specified length, inclusive. Works with arrays.
  maxLength: '※{0}文字以下で入力してください。',
  // max length	Requires the input to have a maximum specified length, inclusive. Works with arrays.
  textLength: '※{0}文字以下で入力してください。',
  // min	Requires entry to have a specified minimum numeric value or Date.
  minValue: '※{param}は、{0}以上の数値を入力してください。',
  // max	Requires entry to have a specified maximum numeric value or Date.
  maxValue: '※{param}は、{0}以下の数値を入力してください。',
  // min, max	Checks if a number or Date is in specified bounds. Min and max are both inclusive.
  between: '※{param}は、{0}以上、{1}以下の数値を入力してください。',
  // none	Accepts only alphabet characters.
  alpha: '※{param}は、半角英字を入力してください。',
  // none	Accepts only alphanumerics.
  alphaNum: '※{param}は、半角英数字を入力してください。',
  // none	Accepts only numerics.
  numeric: '※{param}は、数値を入力してください。',
  // 正の整数
  positiveInteger: '※正の整数を入力してください。',
  // none	Accepts valid email addresses. Keep in mind you still have to carefully verify it on your server, as it is impossible to tell if the address is real without sending verification email.
  email: '※正しいアドレスを入力してください。',
  // none	Accepts valid IPv4 addresses in dotted decimal notation like 127.0.0.1.
  ipAddress: '※{param}は、正しいフォーマットで入力してください。',
  // separator=':'	Accepts valid MAC addresses like 00:ff:11:22:33:44:55. Don't forget to call it macAddress(), as it has optional parameter. You can specify your own separator instead of ':'. Provide empty separator macAddress('') to validate MAC addresses like 00ff1122334455.
  macAddress: '※{param}は、正しいフォーマットで入力してください。',
  // locator *	Checks for equality with a given property.
  sameAs: '※{param}が間違っているようです。',
  // 大小比較
  gt: '※{param} は、{paramCompared} より大きい値を入力してください。',
  ge: '※{param} は、{paramCompared} 以上の値を入力してください。',
  lt: '※{param} は、{paramCompared} より小さい値を入力してください。',
  le: '※{param} は、{paramCompared} 以下の値を入力してください。',
  gtStr: '※{param} は、{paramCompared} より大きい値を入力してください。',
  geStr: '※{param} は、{paramCompared} 以上の値を入力してください。',
  ltStr: '※{param} は、{paramCompared} より小さい値を入力してください。',
  leStr: '※{param} は、{paramCompared} 以下の値を入力してください。',
  // none	Accepts only URLs.
  url: '※{param}は、正しく入力してください。',
  tel: '※数字またはハイフンのみ入力できます。',
  requiredTos: '※{param}に同意して下さい。',

  // エラーメッセージは呼び出しもとで定義する
  anyCondition: '{1}',

  // validators...	Passes when at least one of provided validators passes.
  // or
  // validators...	Passes when all of provided validators passes.
  // and
  // $params, validator	Not really a validator, but a validator modifier. Adds a `$params` object to the provided validator. Can be used on validation functions or even entire nested field validation objects. Useful for creating your own custom validators.
  // withParams
};
