# \Api20100401CredentialListMappingApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_credential_list_mapping**](Api20100401CredentialListMappingApi.md#create_sip_credential_list_mapping) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/CredentialListMappings.json | Create a CredentialListMapping resource for an account.
[**delete_sip_credential_list_mapping**](Api20100401CredentialListMappingApi.md#delete_sip_credential_list_mapping) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/CredentialListMappings/{Sid}.json | Delete a CredentialListMapping resource from an account.
[**fetch_sip_credential_list_mapping**](Api20100401CredentialListMappingApi.md#fetch_sip_credential_list_mapping) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/CredentialListMappings/{Sid}.json | Fetch a single CredentialListMapping resource from an account.
[**list_sip_credential_list_mapping**](Api20100401CredentialListMappingApi.md#list_sip_credential_list_mapping) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/CredentialListMappings.json | Read multiple CredentialListMapping resources from an account.



## create_sip_credential_list_mapping

> models::ApiV2010AccountSipSipDomainSipCredentialListMapping create_sip_credential_list_mapping(account_sid, domain_sid, credential_list_sid)
Create a CredentialListMapping resource for an account.

Create a CredentialListMapping resource for an account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP Domain for which the CredentialList resource will be mapped. | [required] |
**credential_list_sid** | **String** | A 34 character string that uniquely identifies the CredentialList resource to map to the SIP domain. | [required] |

### Return type

[**models::ApiV2010AccountSipSipDomainSipCredentialListMapping**](api.v2010.account.sip.sip_domain.sip_credential_list_mapping.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_credential_list_mapping

> delete_sip_credential_list_mapping(account_sid, domain_sid, sid)
Delete a CredentialListMapping resource from an account.

Delete a CredentialListMapping resource from an account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP Domain that includes the resource to delete. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_credential_list_mapping

> models::ApiV2010AccountSipSipDomainSipCredentialListMapping fetch_sip_credential_list_mapping(account_sid, domain_sid, sid)
Fetch a single CredentialListMapping resource from an account.

Fetch a single CredentialListMapping resource from an account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP Domain that includes the resource to fetch. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountSipSipDomainSipCredentialListMapping**](api.v2010.account.sip.sip_domain.sip_credential_list_mapping.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_credential_list_mapping

> models::ListSipCredentialListMappingResponse list_sip_credential_list_mapping(account_sid, domain_sid, page_size, page, page_token)
Read multiple CredentialListMapping resources from an account.

Read multiple CredentialListMapping resources from an account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP Domain that includes the resource to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipCredentialListMappingResponse**](ListSipCredentialListMappingResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

