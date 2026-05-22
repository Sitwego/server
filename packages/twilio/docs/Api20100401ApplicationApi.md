# \Api20100401ApplicationApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_application**](Api20100401ApplicationApi.md#create_application) | **POST** /2010-04-01/Accounts/{AccountSid}/Applications.json | Create a new application within your account
[**delete_application**](Api20100401ApplicationApi.md#delete_application) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Applications/{Sid}.json | Delete the application by the specified application sid
[**fetch_application**](Api20100401ApplicationApi.md#fetch_application) | **GET** /2010-04-01/Accounts/{AccountSid}/Applications/{Sid}.json | Fetch the application specified by the provided sid
[**list_application**](Api20100401ApplicationApi.md#list_application) | **GET** /2010-04-01/Accounts/{AccountSid}/Applications.json | Retrieve a list of applications representing an application within the requesting account
[**update_application**](Api20100401ApplicationApi.md#update_application) | **POST** /2010-04-01/Accounts/{AccountSid}/Applications/{Sid}.json | Updates the application's properties



## create_application

> models::ApiV2010AccountApplication create_application(account_sid, api_version, voice_url, voice_method, voice_fallback_url, voice_fallback_method, status_callback, status_callback_method, voice_caller_id_lookup, sms_url, sms_method, sms_fallback_url, sms_fallback_method, sms_status_callback, message_status_callback, friendly_name, public_application_connect_enabled)
Create a new application within your account

Create a new application within your account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**api_version** | Option<**String**> | The API version to use to start a new TwiML session. Can be: `2010-04-01` or `2008-08-01`. The default value is the account's default API version. |  |
**voice_url** | Option<**String**> | The URL we should call when the phone number assigned to this application receives a call. |  |
**voice_method** | Option<**String**> | The HTTP method we should use to call `voice_url`. Can be: `GET` or `POST`. |  |
**voice_fallback_url** | Option<**String**> | The URL that we should call when an error occurs retrieving or executing the TwiML requested by `url`. |  |
**voice_fallback_method** | Option<**String**> | The HTTP method we should use to call `voice_fallback_url`. Can be: `GET` or `POST`. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use to call `status_callback`. Can be: `GET` or `POST`. |  |
**voice_caller_id_lookup** | Option<**bool**> | Whether we should look up the caller's caller-ID name from the CNAM database (additional charges apply). Can be: `true` or `false`. |  |
**sms_url** | Option<**String**> | The URL we should call when the phone number receives an incoming SMS message. |  |
**sms_method** | Option<**String**> | The HTTP method we should use to call `sms_url`. Can be: `GET` or `POST`. |  |
**sms_fallback_url** | Option<**String**> | The URL that we should call when an error occurs while retrieving or executing the TwiML from `sms_url`. |  |
**sms_fallback_method** | Option<**String**> | The HTTP method we should use to call `sms_fallback_url`. Can be: `GET` or `POST`. |  |
**sms_status_callback** | Option<**String**> | The URL we should call using a POST method to send status information about SMS messages sent by the application. |  |
**message_status_callback** | Option<**String**> | The URL we should call using a POST method to send message status information to your application. |  |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the new application. It can be up to 64 characters long. |  |
**public_application_connect_enabled** | Option<**bool**> | Whether to allow other Twilio accounts to dial this applicaton using Dial verb. Can be: `true` or `false`. |  |

### Return type

[**models::ApiV2010AccountApplication**](api.v2010.account.application.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_application

> delete_application(account_sid, sid)
Delete the application by the specified application sid

Delete the application by the specified application sid

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Application resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Application resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_application

> models::ApiV2010AccountApplication fetch_application(account_sid, sid)
Fetch the application specified by the provided sid

Fetch the application specified by the provided sid

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Application resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Application resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountApplication**](api.v2010.account.application.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_application

> models::ListApplicationResponse list_application(account_sid, friendly_name, page_size, page, page_token)
Retrieve a list of applications representing an application within the requesting account

Retrieve a list of applications representing an application within the requesting account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Application resources to read. | [required] |
**friendly_name** | Option<**String**> | The string that identifies the Application resources to read. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListApplicationResponse**](ListApplicationResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_application

> models::ApiV2010AccountApplication update_application(account_sid, sid, friendly_name, api_version, voice_url, voice_method, voice_fallback_url, voice_fallback_method, status_callback, status_callback_method, voice_caller_id_lookup, sms_url, sms_method, sms_fallback_url, sms_fallback_method, sms_status_callback, message_status_callback, public_application_connect_enabled)
Updates the application's properties

Updates the application's properties

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Application resources to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Application resource to update. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the resource. It can be up to 64 characters long. |  |
**api_version** | Option<**String**> | The API version to use to start a new TwiML session. Can be: `2010-04-01` or `2008-08-01`. The default value is your account's default API version. |  |
**voice_url** | Option<**String**> | The URL we should call when the phone number assigned to this application receives a call. |  |
**voice_method** | Option<**String**> | The HTTP method we should use to call `voice_url`. Can be: `GET` or `POST`. |  |
**voice_fallback_url** | Option<**String**> | The URL that we should call when an error occurs retrieving or executing the TwiML requested by `url`. |  |
**voice_fallback_method** | Option<**String**> | The HTTP method we should use to call `voice_fallback_url`. Can be: `GET` or `POST`. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use to call `status_callback`. Can be: `GET` or `POST`. |  |
**voice_caller_id_lookup** | Option<**bool**> | Whether we should look up the caller's caller-ID name from the CNAM database (additional charges apply). Can be: `true` or `false`. |  |
**sms_url** | Option<**String**> | The URL we should call when the phone number receives an incoming SMS message. |  |
**sms_method** | Option<**String**> | The HTTP method we should use to call `sms_url`. Can be: `GET` or `POST`. |  |
**sms_fallback_url** | Option<**String**> | The URL that we should call when an error occurs while retrieving or executing the TwiML from `sms_url`. |  |
**sms_fallback_method** | Option<**String**> | The HTTP method we should use to call `sms_fallback_url`. Can be: `GET` or `POST`. |  |
**sms_status_callback** | Option<**String**> | Same as message_status_callback: The URL we should call using a POST method to send status information about SMS messages sent by the application. Deprecated, included for backwards compatibility. |  |
**message_status_callback** | Option<**String**> | The URL we should call using a POST method to send message status information to your application. |  |
**public_application_connect_enabled** | Option<**bool**> | Whether to allow other Twilio accounts to dial this applicaton using Dial verb. Can be: `true` or `false`. |  |

### Return type

[**models::ApiV2010AccountApplication**](api.v2010.account.application.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

