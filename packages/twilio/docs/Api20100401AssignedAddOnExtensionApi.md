# \Api20100401AssignedAddOnExtensionApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_incoming_phone_number_assigned_add_on_extension**](Api20100401AssignedAddOnExtensionApi.md#fetch_incoming_phone_number_assigned_add_on_extension) | **GET** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{ResourceSid}/AssignedAddOns/{AssignedAddOnSid}/Extensions/{Sid}.json | Fetch an instance of an Extension for the Assigned Add-on.
[**list_incoming_phone_number_assigned_add_on_extension**](Api20100401AssignedAddOnExtensionApi.md#list_incoming_phone_number_assigned_add_on_extension) | **GET** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{ResourceSid}/AssignedAddOns/{AssignedAddOnSid}/Extensions.json | Retrieve a list of Extensions for the Assigned Add-on.



## fetch_incoming_phone_number_assigned_add_on_extension

> models::ApiV2010AccountIncomingPhoneNumberIncomingPhoneNumberAssignedAddOnIncomingPhoneNumberAssignedAddOnExtension fetch_incoming_phone_number_assigned_add_on_extension(account_sid, resource_sid, assigned_add_on_sid, sid)
Fetch an instance of an Extension for the Assigned Add-on.

Fetch an instance of an Extension for the Assigned Add-on.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the resource to fetch. | [required] |
**resource_sid** | **String** | The SID of the Phone Number to which the Add-on is assigned. | [required] |
**assigned_add_on_sid** | **String** | The SID that uniquely identifies the assigned Add-on installation. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountIncomingPhoneNumberIncomingPhoneNumberAssignedAddOnIncomingPhoneNumberAssignedAddOnExtension**](api.v2010.account.incoming_phone_number.incoming_phone_number_assigned_add_on.incoming_phone_number_assigned_add_on_extension.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_incoming_phone_number_assigned_add_on_extension

> models::ListIncomingPhoneNumberAssignedAddOnExtensionResponse list_incoming_phone_number_assigned_add_on_extension(account_sid, resource_sid, assigned_add_on_sid, page_size, page, page_token)
Retrieve a list of Extensions for the Assigned Add-on.

Retrieve a list of Extensions for the Assigned Add-on.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the resources to read. | [required] |
**resource_sid** | **String** | The SID of the Phone Number to which the Add-on is assigned. | [required] |
**assigned_add_on_sid** | **String** | The SID that uniquely identifies the assigned Add-on installation. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListIncomingPhoneNumberAssignedAddOnExtensionResponse**](ListIncomingPhoneNumberAssignedAddOnExtensionResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

