# \Api20100401IncomingPhoneNumberApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_incoming_phone_number**](Api20100401IncomingPhoneNumberApi.md#create_incoming_phone_number) | **POST** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers.json | Purchase a phone-number for the account.
[**delete_incoming_phone_number**](Api20100401IncomingPhoneNumberApi.md#delete_incoming_phone_number) | **DELETE** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{Sid}.json | Delete a phone-numbers belonging to the account used to make the request.
[**fetch_incoming_phone_number**](Api20100401IncomingPhoneNumberApi.md#fetch_incoming_phone_number) | **GET** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{Sid}.json | Fetch an incoming-phone-number belonging to the account used to make the request.
[**list_incoming_phone_number**](Api20100401IncomingPhoneNumberApi.md#list_incoming_phone_number) | **GET** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers.json | Retrieve a list of incoming-phone-numbers belonging to the account used to make the request.
[**update_incoming_phone_number**](Api20100401IncomingPhoneNumberApi.md#update_incoming_phone_number) | **POST** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{Sid}.json | Update an incoming-phone-number instance.



## create_incoming_phone_number

> models::ApiV2010AccountIncomingPhoneNumber create_incoming_phone_number(account_sid, api_version, friendly_name, sms_application_sid, sms_fallback_method, sms_fallback_url, sms_method, sms_url, status_callback, status_callback_method, voice_application_sid, voice_caller_id_lookup, voice_fallback_method, voice_fallback_url, voice_method, voice_url, emergency_status, emergency_address_sid, trunk_sid, identity_sid, address_sid, voice_receive_mode, bundle_sid, phone_number, area_code)
Purchase a phone-number for the account.

Purchase a phone-number for the account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**api_version** | Option<**String**> | The API version to use for incoming calls made to the new phone number. The default is `2010-04-01`. |  |
**friendly_name** | Option<**String**> | A descriptive string that you created to describe the new phone number. It can be up to 64 characters long. By default, this is a formatted version of the new phone number. |  |
**sms_application_sid** | Option<**String**> | The SID of the application that should handle SMS messages sent to the new phone number. If an `sms_application_sid` is present, we ignore all of the `sms_*_url` urls and use those set on the application. |  |
**sms_fallback_method** | Option<**String**> | The HTTP method that we should use to call `sms_fallback_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**sms_fallback_url** | Option<**String**> | The URL that we should call when an error occurs while requesting or executing the TwiML defined by `sms_url`. |  |
**sms_method** | Option<**String**> | The HTTP method that we should use to call `sms_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**sms_url** | Option<**String**> | The URL we should call when the new phone number receives an incoming SMS message. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use to call `status_callback`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_application_sid** | Option<**String**> | The SID of the application we should use to handle calls to the new phone number. If a `voice_application_sid` is present, we ignore all of the voice urls and use only those set on the application. Setting a `voice_application_sid` will automatically delete your `trunk_sid` and vice versa. |  |
**voice_caller_id_lookup** | Option<**bool**> | Whether to lookup the caller's name from the CNAM database and post it to your app. Can be: `true` or `false` and defaults to `false`. |  |
**voice_fallback_method** | Option<**String**> | The HTTP method that we should use to call `voice_fallback_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_fallback_url** | Option<**String**> | The URL that we should call when an error occurs retrieving or executing the TwiML requested by `url`. |  |
**voice_method** | Option<**String**> | The HTTP method that we should use to call `voice_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_url** | Option<**String**> | The URL that we should call to answer a call to the new phone number. The `voice_url` will not be called if a `voice_application_sid` or a `trunk_sid` is set. |  |
**emergency_status** | Option<[**models::IncomingPhoneNumberEnumEmergencyStatus**](IncomingPhoneNumberEnumEmergencyStatus.md)> |  |  |
**emergency_address_sid** | Option<**String**> | The SID of the emergency address configuration to use for emergency calling from the new phone number. |  |
**trunk_sid** | Option<**String**> | The SID of the Trunk we should use to handle calls to the new phone number. If a `trunk_sid` is present, we ignore all of the voice urls and voice applications and use only those set on the Trunk. Setting a `trunk_sid` will automatically delete your `voice_application_sid` and vice versa. |  |
**identity_sid** | Option<**String**> | The SID of the Identity resource that we should associate with the new phone number. Some regions require an identity to meet local regulations. |  |
**address_sid** | Option<**String**> | The SID of the Address resource we should associate with the new phone number. Some regions require addresses to meet local regulations. |  |
**voice_receive_mode** | Option<[**models::IncomingPhoneNumberEnumVoiceReceiveMode**](IncomingPhoneNumberEnumVoiceReceiveMode.md)> |  |  |
**bundle_sid** | Option<**String**> | The SID of the Bundle resource that you associate with the phone number. Some regions require a Bundle to meet local Regulations. |  |
**phone_number** | Option<**String**> | The phone number to purchase specified in [E.164](https://www.twilio.com/docs/glossary/what-e164) format.  E.164 phone numbers consist of a + followed by the country code and subscriber number without punctuation characters. For example, +14155551234. |  |
**area_code** | Option<**String**> | The desired area code for your new incoming phone number. Can be any three-digit, US or Canada area code. We will provision an available phone number within this area code for you. **You must provide an `area_code` or a `phone_number`.** (US and Canada only). |  |

### Return type

[**models::ApiV2010AccountIncomingPhoneNumber**](api.v2010.account.incoming_phone_number.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_incoming_phone_number

> delete_incoming_phone_number(account_sid, sid)
Delete a phone-numbers belonging to the account used to make the request.

Delete a phone-numbers belonging to the account used to make the request.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the IncomingPhoneNumber resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the IncomingPhoneNumber resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_incoming_phone_number

> models::ApiV2010AccountIncomingPhoneNumber fetch_incoming_phone_number(account_sid, sid)
Fetch an incoming-phone-number belonging to the account used to make the request.

Fetch an incoming-phone-number belonging to the account used to make the request.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the IncomingPhoneNumber resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the IncomingPhoneNumber resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountIncomingPhoneNumber**](api.v2010.account.incoming_phone_number.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_incoming_phone_number

> models::ListIncomingPhoneNumberResponse list_incoming_phone_number(account_sid, beta, friendly_name, phone_number, origin, page_size, page, page_token)
Retrieve a list of incoming-phone-numbers belonging to the account used to make the request.

Retrieve a list of incoming-phone-numbers belonging to the account used to make the request.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the IncomingPhoneNumber resources to read. | [required] |
**beta** | Option<**bool**> | Whether to include phone numbers new to the Twilio platform. Can be: `true` or `false` and the default is `true`. |  |
**friendly_name** | Option<**String**> | A string that identifies the IncomingPhoneNumber resources to read. |  |
**phone_number** | Option<**String**> | The phone numbers of the IncomingPhoneNumber resources to read. You can specify partial numbers and use '*' as a wildcard for any digit. |  |
**origin** | Option<**String**> | Whether to include phone numbers based on their origin. Can be: `twilio` or `hosted`. By default, phone numbers of all origin are included. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListIncomingPhoneNumberResponse**](ListIncomingPhoneNumberResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_incoming_phone_number

> models::ApiV2010AccountIncomingPhoneNumber update_incoming_phone_number(account_sid, sid, account_sid2, api_version, friendly_name, sms_application_sid, sms_fallback_method, sms_fallback_url, sms_method, sms_url, status_callback, status_callback_method, voice_application_sid, voice_caller_id_lookup, voice_fallback_method, voice_fallback_url, voice_method, voice_url, emergency_status, emergency_address_sid, trunk_sid, voice_receive_mode, identity_sid, address_sid, bundle_sid)
Update an incoming-phone-number instance.

Update an incoming-phone-number instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the IncomingPhoneNumber resource to update.  For more information, see [Exchanging Numbers Between Subaccounts](https://www.twilio.com/docs/iam/api/subaccounts#exchanging-numbers). | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the IncomingPhoneNumber resource to update. | [required] |
**account_sid2** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the IncomingPhoneNumber resource to update.  For more information, see [Exchanging Numbers Between Subaccounts](https://www.twilio.com/docs/iam/api/subaccounts#exchanging-numbers). |  |
**api_version** | Option<**String**> | The API version to use for incoming calls made to the phone number. The default is `2010-04-01`. |  |
**friendly_name** | Option<**String**> | A descriptive string that you created to describe this phone number. It can be up to 64 characters long. By default, this is a formatted version of the phone number. |  |
**sms_application_sid** | Option<**String**> | The SID of the application that should handle SMS messages sent to the number. If an `sms_application_sid` is present, we ignore all of the `sms_*_url` urls and use those set on the application. |  |
**sms_fallback_method** | Option<**String**> | The HTTP method that we should use to call `sms_fallback_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**sms_fallback_url** | Option<**String**> | The URL that we should call when an error occurs while requesting or executing the TwiML defined by `sms_url`. |  |
**sms_method** | Option<**String**> | The HTTP method that we should use to call `sms_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**sms_url** | Option<**String**> | The URL we should call when the phone number receives an incoming SMS message. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use to call `status_callback`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_application_sid** | Option<**String**> | The SID of the application we should use to handle phone calls to the phone number. If a `voice_application_sid` is present, we ignore all of the voice urls and use only those set on the application. Setting a `voice_application_sid` will automatically delete your `trunk_sid` and vice versa. |  |
**voice_caller_id_lookup** | Option<**bool**> | Whether to lookup the caller's name from the CNAM database and post it to your app. Can be: `true` or `false` and defaults to `false`. |  |
**voice_fallback_method** | Option<**String**> | The HTTP method that we should use to call `voice_fallback_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_fallback_url** | Option<**String**> | The URL that we should call when an error occurs retrieving or executing the TwiML requested by `url`. |  |
**voice_method** | Option<**String**> | The HTTP method that we should use to call `voice_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_url** | Option<**String**> | The URL that we should call to answer a call to the phone number. The `voice_url` will not be called if a `voice_application_sid` or a `trunk_sid` is set. |  |
**emergency_status** | Option<[**models::IncomingPhoneNumberEnumEmergencyStatus**](IncomingPhoneNumberEnumEmergencyStatus.md)> |  |  |
**emergency_address_sid** | Option<**String**> | The SID of the emergency address configuration to use for emergency calling from this phone number. |  |
**trunk_sid** | Option<**String**> | The SID of the Trunk we should use to handle phone calls to the phone number. If a `trunk_sid` is present, we ignore all of the voice urls and voice applications and use only those set on the Trunk. Setting a `trunk_sid` will automatically delete your `voice_application_sid` and vice versa. |  |
**voice_receive_mode** | Option<[**models::IncomingPhoneNumberEnumVoiceReceiveMode**](IncomingPhoneNumberEnumVoiceReceiveMode.md)> |  |  |
**identity_sid** | Option<**String**> | The SID of the Identity resource that we should associate with the phone number. Some regions require an identity to meet local regulations. |  |
**address_sid** | Option<**String**> | The SID of the Address resource we should associate with the phone number. Some regions require addresses to meet local regulations. |  |
**bundle_sid** | Option<**String**> | The SID of the Bundle resource that you associate with the phone number. Some regions require a Bundle to meet local Regulations. |  |

### Return type

[**models::ApiV2010AccountIncomingPhoneNumber**](api.v2010.account.incoming_phone_number.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

