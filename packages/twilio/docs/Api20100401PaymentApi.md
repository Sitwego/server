# \Api20100401PaymentApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_payments**](Api20100401PaymentApi.md#create_payments) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Payments.json | create an instance of payments. This will start a new payments session
[**update_payments**](Api20100401PaymentApi.md#update_payments) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Payments/{Sid}.json | update an instance of payments with different phases of payment flows.



## create_payments

> models::ApiV2010AccountCallPayments create_payments(account_sid, call_sid, idempotency_key, status_callback, bank_account_type, charge_amount, currency, description, input, min_postal_code_length, parameter, payment_connector, payment_method, postal_code, security_code, timeout, token_type, valid_card_types, require_matching_inputs, confirmation)
create an instance of payments. This will start a new payments session

create an instance of payments. This will start a new payments session

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**call_sid** | **String** | The SID of the call that will create the resource. Call leg associated with this sid is expected to provide payment information thru DTMF. | [required] |
**idempotency_key** | **String** | A unique token that will be used to ensure that multiple API calls with the same information do not result in multiple transactions. This should be a unique string value per API call and can be a randomly generated. | [required] |
**status_callback** | **String** | Provide an absolute or relative URL to receive status updates regarding your Pay session. Read more about the [expected StatusCallback values](https://www.twilio.com/docs/voice/api/payment-resource#statuscallback) | [required] |
**bank_account_type** | Option<[**models::PaymentsEnumBankAccountType**](PaymentsEnumBankAccountType.md)> |  |  |
**charge_amount** | Option<**f64**> | A positive decimal value less than 1,000,000 to charge against the credit card or bank account. Default currency can be overwritten with `currency` field. Leave blank or set to 0 to tokenize. |  |
**currency** | Option<**String**> | The currency of the `charge_amount`, formatted as [ISO 4127](http://www.iso.org/iso/home/standards/currency_codes.htm) format. The default value is `USD` and all values allowed from the Pay Connector are accepted. |  |
**description** | Option<**String**> | The description can be used to provide more details regarding the transaction. This information is submitted along with the payment details to the Payment Connector which are then posted on the transactions. |  |
**input** | Option<**String**> | A list of inputs that should be accepted. Currently only `dtmf` is supported. All digits captured during a pay session are redacted from the logs. |  |
**min_postal_code_length** | Option<**i32**> | A positive integer that is used to validate the length of the `PostalCode` inputted by the user. User must enter this many digits. |  |
**parameter** | Option<[**serde_json::Value**](SerdeJson__Value.md)> | A single-level JSON object used to pass custom parameters to payment processors. (Required for ACH payments). The information that has to be included here depends on the <Pay> Connector. [Read more](https://www.twilio.com/console/voice/pay-connectors). |  |
**payment_connector** | Option<**String**> | This is the unique name corresponding to the Pay Connector installed in the Twilio Add-ons. Learn more about [<Pay> Connectors](https://www.twilio.com/console/voice/pay-connectors). The default value is `Default`. |  |
**payment_method** | Option<[**models::PaymentsEnumPaymentMethod**](PaymentsEnumPaymentMethod.md)> |  |  |
**postal_code** | Option<**bool**> | Indicates whether the credit card postal code (zip code) is a required piece of payment information that must be provided by the caller. The default is `true`. |  |
**security_code** | Option<**bool**> | Indicates whether the credit card security code is a required piece of payment information that must be provided by the caller. The default is `true`. |  |
**timeout** | Option<**i32**> | The number of seconds that <Pay> should wait for the caller to press a digit between each subsequent digit, after the first one, before moving on to validate the digits captured. The default is `5`, maximum is `600`. |  |
**token_type** | Option<[**models::PaymentsEnumTokenType**](PaymentsEnumTokenType.md)> |  |  |
**valid_card_types** | Option<**String**> | Credit card types separated by space that Pay should accept. The default value is `visa mastercard amex` |  |
**require_matching_inputs** | Option<**String**> | A comma-separated list of payment information fields that require the caller to enter the same value twice for confirmation. Supported values are `payment-card-number`, `expiration-date`, `security-code`, and `postal-code`. |  |
**confirmation** | Option<**String**> | Whether to prompt the caller to confirm their payment information before submitting to the payment gateway. If `true`, the caller will hear the last 4 digits of their card or account number and must press 1 to confirm or 2 to cancel. Default is `false`. |  |

### Return type

[**models::ApiV2010AccountCallPayments**](api.v2010.account.call.payments.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_payments

> models::ApiV2010AccountCallPayments update_payments(account_sid, call_sid, sid, idempotency_key, status_callback, capture, status)
update an instance of payments with different phases of payment flows.

update an instance of payments with different phases of payment flows.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will update the resource. | [required] |
**call_sid** | **String** | The SID of the call that will update the resource. This should be the same call sid that was used to create payments resource. | [required] |
**sid** | **String** | The SID of Payments session that needs to be updated. | [required] |
**idempotency_key** | **String** | A unique token that will be used to ensure that multiple API calls with the same information do not result in multiple transactions. This should be a unique string value per API call and can be a randomly generated. | [required] |
**status_callback** | **String** | Provide an absolute or relative URL to receive status updates regarding your Pay session. Read more about the [Update](https://www.twilio.com/docs/voice/api/payment-resource#statuscallback-update) and [Complete/Cancel](https://www.twilio.com/docs/voice/api/payment-resource#statuscallback-cancelcomplete) POST requests. | [required] |
**capture** | Option<[**models::PaymentsEnumCapture**](PaymentsEnumCapture.md)> |  |  |
**status** | Option<[**models::PaymentsEnumStatus**](PaymentsEnumStatus.md)> |  |  |

### Return type

[**models::ApiV2010AccountCallPayments**](api.v2010.account.call.payments.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

