# \Api20100401AssignedAddOnApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_incoming_phone_number_assigned_add_on**](Api20100401AssignedAddOnApi.md#create_incoming_phone_number_assigned_add_on) | **POST** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{ResourceSid}/AssignedAddOns.json | Assign an Add-on installation to the Number specified.
[**delete_incoming_phone_number_assigned_add_on**](Api20100401AssignedAddOnApi.md#delete_incoming_phone_number_assigned_add_on) | **DELETE** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{ResourceSid}/AssignedAddOns/{Sid}.json | Remove the assignment of an Add-on installation from the Number specified.
[**fetch_incoming_phone_number_assigned_add_on**](Api20100401AssignedAddOnApi.md#fetch_incoming_phone_number_assigned_add_on) | **GET** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{ResourceSid}/AssignedAddOns/{Sid}.json | Fetch an instance of an Add-on installation currently assigned to this Number.
[**list_incoming_phone_number_assigned_add_on**](Api20100401AssignedAddOnApi.md#list_incoming_phone_number_assigned_add_on) | **GET** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/{ResourceSid}/AssignedAddOns.json | Retrieve a list of Add-on installations currently assigned to this Number.



## create_incoming_phone_number_assigned_add_on

> models::ApiV2010AccountIncomingPhoneNumberIncomingPhoneNumberAssignedAddOn create_incoming_phone_number_assigned_add_on(account_sid, resource_sid, installed_add_on_sid)
Assign an Add-on installation to the Number specified.

Assign an Add-on installation to the Number specified.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**resource_sid** | **String** | The SID of the Phone Number to assign the Add-on. | [required] |
**installed_add_on_sid** | **String** | The SID that identifies the Add-on installation. | [required] |

### Return type

[**models::ApiV2010AccountIncomingPhoneNumberIncomingPhoneNumberAssignedAddOn**](api.v2010.account.incoming_phone_number.incoming_phone_number_assigned_add_on.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_incoming_phone_number_assigned_add_on

> delete_incoming_phone_number_assigned_add_on(account_sid, resource_sid, sid)
Remove the assignment of an Add-on installation from the Number specified.

Remove the assignment of an Add-on installation from the Number specified.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the resources to delete. | [required] |
**resource_sid** | **String** | The SID of the Phone Number to which the Add-on is assigned. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_incoming_phone_number_assigned_add_on

> models::ApiV2010AccountIncomingPhoneNumberIncomingPhoneNumberAssignedAddOn fetch_incoming_phone_number_assigned_add_on(account_sid, resource_sid, sid)
Fetch an instance of an Add-on installation currently assigned to this Number.

Fetch an instance of an Add-on installation currently assigned to this Number.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the resource to fetch. | [required] |
**resource_sid** | **String** | The SID of the Phone Number to which the Add-on is assigned. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountIncomingPhoneNumberIncomingPhoneNumberAssignedAddOn**](api.v2010.account.incoming_phone_number.incoming_phone_number_assigned_add_on.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_incoming_phone_number_assigned_add_on

> models::ListIncomingPhoneNumberAssignedAddOnResponse list_incoming_phone_number_assigned_add_on(account_sid, resource_sid, page_size, page, page_token)
Retrieve a list of Add-on installations currently assigned to this Number.

Retrieve a list of Add-on installations currently assigned to this Number.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the resources to read. | [required] |
**resource_sid** | **String** | The SID of the Phone Number to which the Add-on is assigned. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListIncomingPhoneNumberAssignedAddOnResponse**](ListIncomingPhoneNumberAssignedAddOnResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

